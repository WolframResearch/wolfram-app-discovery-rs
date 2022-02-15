use std::{
    collections::HashMap, ffi::c_void, path::PathBuf, ptr::null_mut as nullptr,
    str::FromStr,
};

use windows::Win32::{
    Foundation::{
        BOOL, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, MAX_PATH,
        PWSTR,
    },
    Storage::{
        FileSystem::{Wow64DisableWow64FsRedirection, Wow64RevertWow64FsRedirection},
        Packaging::Appx::{
            ClosePackageInfo, GetPackageInfo, GetPackagesByPackageFamily,
            GetStagedPackageOrigin, OpenPackageInfoByFullName, PackageOrigin,
            PackageOrigin_DeveloperSigned, PackageOrigin_DeveloperUnsigned,
            PackageOrigin_Inbox, PackageOrigin_LineOfBusiness, PackageOrigin_Store,
            PackageOrigin_Unknown, PackageOrigin_Unsigned, APPX_PACKAGE_ARCHITECTURE,
            APPX_PACKAGE_ARCHITECTURE_ARM, APPX_PACKAGE_ARCHITECTURE_ARM64,
            APPX_PACKAGE_ARCHITECTURE_X64, APPX_PACKAGE_ARCHITECTURE_X86, PACKAGE_INFO,
            PACKAGE_INFORMATION_FULL, _PACKAGE_INFO_REFERENCE,
        },
    },
    System::{
        Diagnostics::Debug::{
            PROCESSOR_ARCHITECTURE, PROCESSOR_ARCHITECTURE_AMD64,
            PROCESSOR_ARCHITECTURE_ARM, PROCESSOR_ARCHITECTURE_INTEL,
        },
        Registry::{
            RegCloseKey, RegEnumKeyW, RegGetValueW, RegOpenKeyExA, RegOpenKeyExW,
            RegQueryInfoKeyW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ,
            KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_SAM_FLAGS, RRF_RT_REG_DWORD,
            RRF_RT_REG_SZ,
        },
        SystemInformation::{GetNativeSystemInfo, SYSTEM_INFO},
        SystemServices::PROCESSOR_ARCHITECTURE_ARM64,
        Threading::{GetCurrentProcess, IsWow64Process},
    },
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{AppVersion, Error, WolframApp, WolframAppType};

//======================================
// Public Interface
//======================================

pub fn discover_all() -> Vec<WolframApp> {
    unsafe { load_apps_from_registry() }
}

//======================================
// Implementation
//======================================


#[derive(Debug, Default)]
struct WolframAppBuilder {
    app_name: Option<String>,
    app_version: Option<AppVersion>,

    app_type: Option<WolframAppType>,

    system_id: Option<String>,

    id: Option<String>,

    installation_directory: Option<PathBuf>,

    language_tag: Option<String>,

    executable_path: Option<PathBuf>,

    digitally_signed: Option<bool>,

    origin: Option<Origin>,
}

#[non_exhaustive]
#[derive(Debug)]
enum Origin {
    Sideloaded,
    Store,
    Unknown,
}

impl WolframAppBuilder {
    fn finish(self) -> Result<WolframApp, ()> {
        let WolframAppBuilder {
            app_name,
            app_version,
            app_type,
            installation_directory,
            executable_path,
            // TODO: Expose these fields?
            system_id: _,
            id: _,
            language_tag: _,
            digitally_signed: _,
            origin: _,
        } = self;

        Ok(WolframApp {
            app_name: app_name.ok_or(())?,
            app_version: app_version.ok_or(())?,
            app_type: app_type.ok_or(())?,

            // TODO: Is this always correct on Windows?
            app_directory: installation_directory.ok_or(())?,
            app_executable: executable_path,

            embedded_player: None,
        }
        .set_engine_embedded_player()
        .map_err(|_| ())?)
    }
}

impl AppVersion {
    fn parse_windows(version: &str) -> Result<Self, Error> {
        fn parse(s: &str) -> Result<u32, Error> {
            u32::from_str(s).map_err(|err| {
                Error(format!(
                    "invalid application version number component: '{}': {}",
                    s, err
                ))
            })
        }

        let components: Vec<&str> = version.split(".").collect();

        let app_version = match components.as_slice() {
            // 4 components: major.minor.revision.minor_revision
            [major, minor, revision, minor_revision] => AppVersion {
                major: parse(major)?,
                minor: parse(minor)?,
                revision: parse(revision)?,

                minor_revision: Some(parse(minor_revision)?),
                build_code: None,
            },
            // 3 components: major.minor.revision
            [major, minor, revision] => AppVersion {
                major: parse(major)?,
                minor: parse(minor)?,
                revision: parse(revision)?,

                minor_revision: None,
                build_code: None,
            },
            _ => {
                return Err(Error(format!(
                    "unexpected application version number format: {}",
                    version
                )))
            },
        };

        Ok(app_version)
    }
}

type DWORD = u32;
type WCHAR = u16;

const PRODUCTS: &[&str] = &[
    "Wolfram.Mathematica_ztr62y9da0nfr",
    "Wolfram.Desktop_ztr62y9da0nfr",
    "Wolfram.Player_ztr62y9da0nfr",
    "Wolfram.FinancePlatform_ztr62y9da0nfr",
    "Wolfram.ProgrammingLab_ztr62y9da0nfr",
    "Wolfram.AlphaNotebook_ztr62y9da0nfr",
    "Wolfram.Engine_ztr62y9da0nfr",
];

#[rustfmt::skip]
static PACKAGE_FAMILY_TO_PRODUCT_NAMES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from_iter([
        ("Wolfram.Mathematica",                    "Wolfram Mathematica"),
        ("Wolfram.Mathematica.Documentation",      "Wolfram Mathematica Documentation"),
        ("Wolfram.Desktop",                        "Wolfram Desktop"),
        ("Wolfram.Desktop.Documentation",          "Wolfram Desktop Documentation"),
        ("Wolfram.Player",                         "Wolfram Player"),
        ("Wolfram.FinancePlatform",                "Wolfram Finance Platform"),
        ("Wolfram.FinancePlatform.Documentation",  "Wolfram Finance Platform Documentation"),
        ("Wolfram.ProgrammingLab",                 "Wolfram Programming Lab"),
        ("Wolfram.ProgrammingLab.Documentation",   "Wolfram Programming Lab Documentation"),
        ("Wolfram.AlphaNotebook",                  "Wolfram|Alpha Notebook Edition"),
        ("Wolfram.AlphaNotebook.Documentation",    "Wolfram|Alpha Notebook Edition Documentation"),
        ("Wolfram.Engine",                         "Wolfram Engine"),
    ])
});

#[rustfmt::skip]
static PACKAGE_FAMILY_TO_APP_TYPE: Lazy<HashMap<&str, WolframAppType>> = Lazy::new(|| {
    // FIXME: How should documentation installations be handled? Modeling them as
    //        independent `WolframApp` instances doesn't seem quite optimal, since
    //        nominally most Wolfram apps provide a copy of the Wolfram Language
    //        runtime, which documentation does not.
    HashMap::from_iter([
        ("Wolfram.Mathematica",                    WolframAppType::Mathematica),
        // ("Wolfram.Mathematica.Documentation",      PRODUCT_MATHEMATICA),
        ("Wolfram.Desktop",                        WolframAppType::Desktop),
        // ("Wolfram.Desktop.Documentation",          PRODUCT_WOLFRAMDESKTOP),
        ("Wolfram.Player",                         WolframAppType::Player),
        ("Wolfram.FinancePlatform",                WolframAppType::FinancePlatform),
        // ("Wolfram.FinancePlatform.Documentation",  PRODUCT_WOLFRAMFINANCE),
        ("Wolfram.ProgrammingLab",                 WolframAppType::ProgrammingLab),
        // ("Wolfram.ProgrammingLab.Documentation",   PRODUCT_WOLFRAMPROGLAB),
        ("Wolfram.AlphaNotebook",                  WolframAppType::WolframAlphaNotebookEdition),
        // ("Wolfram.AlphaNotebook.Documentation",    PRODUCT_WOLFRAMALPHANB),
        ("Wolfram.Engine",                         WolframAppType::Engine)
    ])
});

fn parse_build_number(build_number: &str) -> Option<u32> {
    let regex = Regex::new(
        "^[a-zA-Z]-[a-zA-Z0-9]+-[a-zA-Z]+(?:\\.[-a-zA-Z]+)?\\.[0-9]+\\.[0-9]+\\.[0-9]+\\.([0-9]+)$"
        //                                                                  build number ^^^^^^^^
    ).unwrap();

    if let Some(captures) = regex.captures(&build_number) {
        return DWORD::from_str(&captures[1]).ok();
    } else if let Ok(number) = DWORD::from_str(&build_number) {
        return Some(number);
    } else {
        None
    }
}

fn win_is_wow_process() -> bool {
    // #if _M_X64 || _M_ARM64
    if cfg!(any(target_arch = "x86_64", target_arch = "aarch64")) {
        return false;
    } else {
        let mut is_wow: BOOL = BOOL::from(false);

        unsafe {
            IsWow64Process(GetCurrentProcess(), &mut is_wow);
        }

        return is_wow.as_bool();
    }
}

fn win_host_system_id() -> String {
    let PROCESSOR_ARCHITECTURE(arch) = unsafe {
        let mut info: SYSTEM_INFO = SYSTEM_INFO::default();
        GetNativeSystemInfo(&mut info);

        info.Anonymous.Anonymous.wProcessorArchitecture
    };

    let arch = u32::from(arch);

    let system_id = match arch {
        _ if arch == u32::from(PROCESSOR_ARCHITECTURE_ARM.0) => "Windows-ARM",
        PROCESSOR_ARCHITECTURE_ARM64 => "Windows-ARM64",
        _ if arch == u32::from(PROCESSOR_ARCHITECTURE_AMD64.0) => "Windows-x86-64",
        _ if arch == u32::from(PROCESSOR_ARCHITECTURE_INTEL.0) => "Windows",
        _ => "Windows",
    };

    String::from(system_id)
}

unsafe fn load_app_from_registry(
    build_key: HKEY,
    system_id: &str,
    build_number: *const WCHAR,
) -> Result<WolframApp, ()> {
    let mut app_builder: WolframAppBuilder = Default::default();

    app_builder.system_id = Some(String::from(system_id));

    let is_wow_proc = win_is_wow_process();

    let mut enabled: DWORD = 0;
    let mut product: DWORD = 0;
    let mut caps: DWORD = 0;
    let mut size: DWORD;

    let build_number = utf16_ptr_to_string(build_number);

    let this_build: DWORD = match parse_build_number(&build_number) {
        Some(build_number) => build_number,
        None => return Err(()),
    };

    if this_build == 0 {
        return Err(());
    }

    // PRE_COMMIT
    // app_builder.setBuildNumber(this_build);

    size = std::mem::size_of::<DWORD>() as u32;
    if RegGetValueW(
        build_key,
        PWSTR(nullptr()),
        "Caps",
        RRF_RT_REG_DWORD,
        nullptr(),
        &mut caps as *mut DWORD as *mut c_void,
        &mut size,
    ) != ERROR_SUCCESS
    {
        return Err(());
    }

    // PRE_COMMIT:
    // app_builder.setCaps(caps);

    size = std::mem::size_of::<DWORD>() as u32;
    if RegGetValueW(
        build_key,
        PWSTR(nullptr()),
        "ProductType",
        RRF_RT_REG_DWORD,
        nullptr(),
        (&mut product) as *mut DWORD as *mut c_void,
        &mut size,
    ) != ERROR_SUCCESS
    {
        return Err(());
    }

    app_builder.app_type = WolframAppType::from_windows_product_type(product);

    if let Some(id) = reg_get_value_string(build_key, "CLSID") {
        app_builder.id = Some(id);
    }

    if let Some(dir) = reg_get_value_string(build_key, "InstallationDirectory") {
        app_builder.installation_directory = Some(PathBuf::from(dir));
    }

    if let Some(exec_path) = reg_get_value_string(build_key, "ExecutablePath") {
        let exec_path = PathBuf::from(exec_path);

        app_builder.executable_path = Some(exec_path.clone());

        // If `installation_directory` is not set but `executable_path` is, derive
        // the installation directory from the executable path.
        if app_builder.installation_directory.is_none() && exec_path.exists() {
            let install_dir = exec_path.parent().unwrap().to_path_buf();
            app_builder.installation_directory = Some(install_dir);
        }
    }

    {
        let has_exec_path = match app_builder.executable_path {
            None => false,
            Some(ref path) => path.exists(),
        };

        let has_install_dir = match app_builder.installation_directory {
            None => false,
            Some(ref path) => path.exists(),
        };

        if !has_exec_path && !has_install_dir {
            return Err(());
        }
    }

    app_builder.language_tag = Some(
        reg_get_value_string(build_key, "Language").unwrap_or_else(|| String::from("en")),
    );

    app_builder.app_name = match reg_get_value_string(build_key, "ProductName") {
        name @ Some(_) => name,
        None => return Err(()),
    };

    if let Some(version_string) = reg_get_value_string(build_key, "ProductVersion") {
        match AppVersion::parse_windows(&version_string) {
            Ok(version) => {
                app_builder.app_version = Some(version);
            },
            Err(_) => {
                // TODO: Generate an error here?
            },
        }
    }

    if RegGetValueW(
        build_key,
        PWSTR(nullptr()),
        "Version",
        RRF_RT_REG_DWORD,
        nullptr(),
        &mut enabled as *mut DWORD as *mut c_void,
        &mut size,
    ) == ERROR_SUCCESS
    {
        let [major, minor, revision, minor_revision] = enabled.to_be_bytes();

        if (major, minor, revision, minor_revision) == (0, 0, 0, 0) {
            // TODO: Does this zero version number appear only in Prototype builds?

            // Don't set the version number based on this registry value.
            crate::warning(&format!(
                "application registry key \"Version\" value is 0.0.0.0  (at: {:?})",
                app_builder.installation_directory
            ));
        } else {
            app_builder.app_version = Some(AppVersion {
                major: u32::from(major),
                minor: u32::from(minor),
                revision: u32::from(revision),
                minor_revision: Some(u32::from(minor_revision)),

                build_code: None,
            });
        }
    }

    if !app_builder.app_version.is_some() {
        let version_file: PathBuf = app_builder
            .installation_directory
            .clone()
            .unwrap()
            .join(".VersionID");

        let mut orginal_value: *mut c_void = nullptr();

        if is_wow_proc {
            Wow64DisableWow64FsRedirection(&mut orginal_value);
        }
        let result = std::fs::read_to_string(&version_file);
        if is_wow_proc {
            Wow64RevertWow64FsRedirection(orginal_value);
        }

        if let Ok(version_string) = result {
            if let Ok(app_version) = AppVersion::parse_windows(&version_string) {
                app_builder.app_version = Some(app_version);
            }
        }
    }

    if app_builder.app_version.is_none() {
        return Err(());
    }

    return app_builder.finish();
}

unsafe fn load_app_from_package_info(
    package_info: &PACKAGE_INFO,
    app_builder: &mut WolframAppBuilder,
) {
    app_builder.id = Some(utf16_ptr_to_string(package_info.packageFullName.0));

    // PRE_COMMIT
    // app_builder.setFullVersion(package_info.packageId.version.Anonymous.Version);

    let package_id_name = utf16_ptr_to_string(package_info.packageId.name.0);

    {
        // because we cannot get our hands on the display name...
        let mut product_title = String::from("Unknown");

        if let Some(iter) = PACKAGE_FAMILY_TO_PRODUCT_NAMES.get(package_id_name.as_str())
        {
            let app_version = app_builder.app_version.clone().unwrap();

            let iter: &str = iter;
            product_title = iter.to_owned() + " " + &app_version.major().to_string();

            if app_version.minor() != 0 {
                product_title += &format!(".{}", &app_version.minor());
            }
        }

        app_builder.app_name = Some(product_title);
    }

    if let Some(app_type) = PACKAGE_FAMILY_TO_APP_TYPE.get(package_id_name.as_str()) {
        app_builder.app_type = Some(app_type.clone());
    } else {
        // PRE_COMMIT
        // app_builder.setProductType(PRODUCT_READER);
    }

    let system_id = match APPX_PACKAGE_ARCHITECTURE(
        package_info
            .packageId
            .processorArchitecture
            .try_into()
            .unwrap(),
    ) {
        APPX_PACKAGE_ARCHITECTURE_ARM => "Windows-ARM",
        APPX_PACKAGE_ARCHITECTURE_ARM64 => "Windows-ARM64",
        APPX_PACKAGE_ARCHITECTURE_X86 => "Windows",
        APPX_PACKAGE_ARCHITECTURE_X64 => "Windows-x86-64",
        _ => "Unknown",
    };

    app_builder.system_id = Some(String::from(system_id));

    let mut raw_origin = PackageOrigin::default();

    #[allow(non_upper_case_globals)]
    if GetStagedPackageOrigin(package_info.packageFullName, &mut raw_origin)
        == ERROR_SUCCESS.0 as i32
    {
        let origin = match raw_origin {
            PackageOrigin_DeveloperUnsigned
            | PackageOrigin_DeveloperSigned
            | PackageOrigin_Inbox
            | PackageOrigin_LineOfBusiness
            | PackageOrigin_Unsigned => Origin::Sideloaded,
            PackageOrigin_Store => Origin::Store,
            PackageOrigin_Unknown | _ => Origin::Unknown,
        };

        app_builder.origin = Some(origin);

        match raw_origin {
            PackageOrigin_Inbox
            | PackageOrigin_DeveloperSigned
            | PackageOrigin_LineOfBusiness
            | PackageOrigin_Store => {
                app_builder.digitally_signed = Some(true);
            },

            PackageOrigin_DeveloperUnsigned
            | PackageOrigin_Unknown
            | PackageOrigin_Unsigned
            | _ => {
                app_builder.digitally_signed = Some(false);
            },
        }
    }

    // TODO: Set language tag to None in this case?
    app_builder.language_tag = Some(String::from("Neutral"));
    app_builder.installation_directory =
        Some(PathBuf::from(utf16_ptr_to_string(package_info.path.0)));

    // PRE_COMMIT
    // app_builder.setBuildNumber(ReadCreationIDFileFromLayout(package_info.path));
}

fn merge_user_installed_packages(apps: &mut Vec<WolframApp>) {
    for product in PRODUCTS {
        let product_apps = unsafe { get_user_packages(product) };
        apps.extend(product_apps);
    }
}

unsafe fn get_user_packages(product: &str) -> Vec<WolframApp> {
    let mut count: u32 = 0;
    let mut buffer_length: u32 = 0;

    let error: i32 = GetPackagesByPackageFamily(
        product,
        &mut count,
        nullptr(),
        &mut buffer_length,
        PWSTR(nullptr()),
    );

    if count == 0 || error != ERROR_INSUFFICIENT_BUFFER.0 as i32 {
        return vec![];
    }

    // let buffer: PWSTR = malloc(size_of::<WCHAR>() * buffer_length) as *mut WCHAR;
    let mut buffer_vec: Vec<u16> =
        Vec::with_capacity(usize::try_from(buffer_length).unwrap());
    let buffer: *mut u16 = buffer_vec.as_mut_ptr();

    // let packageFullNames: *mut PWSTR = malloc(size_of::<PWSTR>() * count) as *mut PWSTR;
    let mut package_full_names: Vec<PWSTR> =
        Vec::with_capacity(usize::try_from(count).unwrap());

    if GetPackagesByPackageFamily(
        product,
        &mut count,
        package_full_names.as_mut_ptr(),
        &mut buffer_length,
        PWSTR(buffer),
    ) != ERROR_SUCCESS.0 as i32
    {
        return vec![];
    }

    package_full_names.set_len(usize::try_from(count).unwrap());

    let mut apps = Vec::new();

    for package_full_name in package_full_names {
        let mut piref: *mut _PACKAGE_INFO_REFERENCE = nullptr();

        if OpenPackageInfoByFullName(package_full_name, 0, &mut piref)
            != ERROR_SUCCESS.0 as i32
        {
            continue;
        }

        let mut app_builder = WolframAppBuilder::default();

        let mut pack_length: u32 = 0;
        let mut pack_count: u32 = 0;

        if GetPackageInfo(
            piref,
            PACKAGE_INFORMATION_FULL,
            &mut pack_length,
            nullptr(),
            &mut pack_count,
        ) == ERROR_INSUFFICIENT_BUFFER.0 as i32
        {
            let mut pack_info_buffer: Vec<u8> =
                Vec::with_capacity(usize::try_from(pack_length).unwrap());

            if GetPackageInfo(
                piref,
                PACKAGE_INFORMATION_FULL,
                &mut pack_length,
                pack_info_buffer.as_mut_ptr(),
                &mut pack_count,
            ) == ERROR_SUCCESS.0 as i32
            {
                // PRE_COMMIT: Is this even close to safe?
                let package_info: *const PACKAGE_INFO =
                    pack_info_buffer.as_ptr() as *const PACKAGE_INFO;

                load_app_from_package_info(&*package_info, &mut app_builder);

                // PRE_COMMIT
                // UpdateCapsFromApplicationIds(piref, package_info, app_builder);
            }
        }

        // UINT32 optPackLength = 0, optPackCount = 0;
        // if (GetPackageInfo(piref, PACKAGE_FILTER_OPTIONAL, &optPackLength, nullptr, &optPackCount)
        // 	== ERROR_INSUFFICIENT_BUFFER)
        // {
        // 	LPBYTE optPackInfoBuffer = (LPBYTE)malloc(optPackLength);
        // 	if (GetPackageInfo(piref, PACKAGE_FILTER_OPTIONAL, &optPackLength, optPackInfoBuffer, &optPackCount)
        // 		== ERROR_SUCCESS)
        // 	{
        // 		std::vector<Wolfram::Apps::InstalledProduct> theOptionalProducts;
        // 		for (UINT32 i = 0; i < optPackCount; i++)
        // 		{
        // 			PACKAGE_INFO_REFERENCE optpiref = nullptr;
        // 			PACKAGE_INFO* package_info = (PACKAGE_INFO*)optPackInfoBuffer;
        // 			Wolfram::Apps::InstalledProduct theOptionalProduct;

        // 			if (OpenPackageInfoByFullName(package_info->packageFullName, 0, &optpiref) == ERROR_SUCCESS)
        // 			{
        // 				LoadInstalledProductInfoFromPackageInfo(package_info, theOptionalProduct);
        // 				UpdateCapsFromApplicationIds(optpiref, package_info, theOptionalProduct);
        // 				cpi(optpiref);
        // 			}

        // 			theOptionalProducts.push(theOptionalProduct);
        // 		}

        // 		app_builder.setOptionalPackages(theOptionalProducts);
        // 	}

        // 	free(optPackInfoBuffer);
        // }

        apps.push(app_builder.finish().expect("PRE_COMMIT"));

        ClosePackageInfo(piref);
    }

    apps
}


unsafe fn load_apps_from_registry() -> Vec<WolframApp> {
    let mut installations: Vec<WolframApp> = Vec::new();

    let mut the_root_key: HKEY = HKEY(0);
    let mut the_alt_root_key: HKEY = HKEY(0);
    let mut the_user_key: HKEY = HKEY(0);

    let is_wow: bool = win_is_wow_process();
    let mut needs_alt: bool = true;

    let mut access_type: REG_SAM_FLAGS = KEY_READ | KEY_WOW64_64KEY;
    let mut alt_access_type: REG_SAM_FLAGS = KEY_READ | KEY_WOW64_32KEY;

    let mut num_root_keys: DWORD = 0;
    let mut num_alt_root_keys: DWORD = 0;
    let mut num_user_keys: DWORD = 0;

    let host_system_id: String = win_host_system_id();

    // #if _M_X64 || _M_ARM64
    if cfg!(any(target_arch = "x86_64", target_arch = "aarch64")) {
        if !is_wow {
            access_type = KEY_READ;
            alt_access_type = KEY_READ;
            needs_alt = false;
        }
    }

    //  64-bit key on WIN64 || is_wow, 32-bit key on WIN32 && !is_wow
    RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        "Software\\Wolfram Research\\Installations",
        0,
        access_type,
        &mut the_root_key,
    );
    RegOpenKeyExA(
        HKEY_CURRENT_USER,
        "Software\\Wolfram Research\\Installations",
        0,
        access_type,
        &mut the_user_key,
    );

    if needs_alt {
        // 32-bit key on WIN64 || is_wow
        RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            "Software\\Wolfram Research\\Installations",
            0,
            alt_access_type,
            &mut the_alt_root_key,
        );
    }

    if the_root_key != HKEY(0) {
        RegQueryInfoKeyW(
            the_root_key,
            PWSTR(nullptr()),
            nullptr(),
            nullptr(),
            &mut num_root_keys,
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
        );
    }
    if needs_alt && the_alt_root_key != HKEY(0) {
        RegQueryInfoKeyW(
            the_alt_root_key,
            PWSTR(nullptr()),
            nullptr(),
            nullptr(),
            &mut num_alt_root_keys,
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
        );
    }
    if the_user_key != HKEY(0) {
        RegQueryInfoKeyW(
            the_user_key,
            PWSTR(nullptr()),
            nullptr(),
            nullptr(),
            &mut num_user_keys,
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
            nullptr(),
        );
    }

    installations.reserve(
        usize::try_from(num_root_keys + num_alt_root_keys + num_user_keys + 1).unwrap(),
    );

    let mut load_products_from_registry_key =
        |the_key: HKEY, access_type: REG_SAM_FLAGS, system_id: &str| {
            let mut build_number: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let mut index: DWORD = 0;

            while RegEnumKeyW(the_key, index, PWSTR(build_number.as_mut_ptr()), MAX_PATH)
                != ERROR_NO_MORE_ITEMS
            {
                let mut build_key: HKEY = HKEY(0);
                if RegOpenKeyExW(
                    the_key,
                    PWSTR(build_number.as_ptr()),
                    0,
                    access_type,
                    &mut build_key,
                ) == ERROR_SUCCESS
                {
                    if let Ok(app) = load_app_from_registry(
                        build_key,
                        system_id,
                        build_number.as_ptr(),
                    ) {
                        installations.push(app);
                    }

                    RegCloseKey(build_key);
                }

                index += 1;
            }
        };

    if the_root_key != HKEY(0) {
        load_products_from_registry_key(
            the_root_key,
            access_type,
            if needs_alt {
                &host_system_id
            } else {
                "Windows"
            },
        );
        RegCloseKey(the_root_key);
    }

    if needs_alt && the_alt_root_key != HKEY(0) {
        load_products_from_registry_key(the_alt_root_key, alt_access_type, "Windows");
        RegCloseKey(the_alt_root_key);
    }

    if the_user_key != HKEY(0) {
        load_products_from_registry_key(
            the_user_key,
            access_type,
            if needs_alt {
                &host_system_id
            } else {
                "Windows"
            },
        );
        RegCloseKey(the_user_key);
    }

    merge_user_installed_packages(&mut installations);

    return installations;
}

impl WolframAppType {
    /// Construct a [`WolframAppType`] from the Windows registry `"ProductType"` field
    /// associated with an application.
    #[rustfmt::skip]
    fn from_windows_product_type(id: u32) -> Option<Self> {
        use WolframAppType::*;

        // const UNIVERSAL: u32        = 0xFFFFFFFF;
        const MATHEMATICA: u32      = 1 << 28; //(0x10000000)
        const DESKTOP: u32          = 1 << 27; //(0x08000000)
        const PROGRAMMING_LAB: u32  = 1 << 26; //(0x04000000)
        const FINANCE_PLATFORM: u32 = 1 << 25; //(0x02000000)
        const ALPHA_NB_EDITION: u32 = 1 << 24; //(0x01000000)
        const ENGINE: u32           = 1 << 15; //(0x00008000)
        const PLAYER_PRO: u32       = 1 << 14; //(0x00004000)
        const PLAYER: u32           = 1 << 1;  //(0x00000002)
        // const READER: u32           = 1;
        // const NONE: u32             = 0;

        let app_type = match id {
            MATHEMATICA => Mathematica,
            DESKTOP => Desktop,
            PROGRAMMING_LAB => ProgrammingLab,
            FINANCE_PLATFORM => FinancePlatform,
            ALPHA_NB_EDITION => WolframAlphaNotebookEdition,
            ENGINE => Engine,
            PLAYER_PRO => PlayerPro,
            PLAYER => Player,
            _ => return None,
        };

        Some(app_type)
    }
}

//======================================
// Utilities
//======================================

unsafe fn utf16_ptr_to_string(str: *const u16) -> String {
    if str.is_null() {
        return String::new();
    }

    // Find the offset of the NULL byte.
    let len: usize = {
        let mut end = str;
        while *end != 0 {
            end = end.add(1);
        }

        usize::try_from(end.offset_from(str)).unwrap()
    };

    let slice: &[u16] = std::slice::from_raw_parts(str, len);

    String::from_utf16(slice).expect("unable to convert string to UTF-16")
}

unsafe fn reg_get_value_string(key: HKEY, name: &str) -> Option<String> {
    let mut size_in_bytes: DWORD = 0;

    if RegGetValueW(
        key,
        PWSTR(nullptr()),
        name,
        RRF_RT_REG_SZ,
        nullptr(),
        nullptr(),
        &mut size_in_bytes,
    ) != ERROR_SUCCESS
    {
        return None;
    }

    let size_in_elements = size_in_bytes / (std::mem::size_of::<WCHAR>() as DWORD);

    let mut buffer: Vec<WCHAR> = vec![0; usize::try_from(size_in_elements).unwrap()];

    if RegGetValueW(
        key,
        PWSTR(nullptr()),
        name,
        RRF_RT_REG_SZ,
        nullptr(),
        buffer.as_mut_ptr() as *mut c_void,
        &mut size_in_bytes,
    ) != ERROR_SUCCESS
    {
        return None;
    }

    Some(utf16_ptr_to_string(buffer.as_ptr()))
}
