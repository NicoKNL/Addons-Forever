//! Embed a Windows application manifest (PerMonitorV2 DPI awareness + common
//! controls + asInvoker) so the GUI scales correctly on high-DPI displays.
//!
//! Toolchain resolution (first hit wins): `$WINDRES`, `x86_64-w64-mingw32-windres`
//! / `i686-w64-mingw32-windres` on PATH, `windres`, MSVC `rc.exe`. If none is
//! found the build succeeds without the manifest (DPI falls back to system
//! scaling) — never breaks cross builds.

use std::path::Path;
use std::process::Command;

const MANIFEST: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0"
          xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <assemblyIdentity type="win32" name="AddonsForever" version="0.2.0.0" processorArchitecture="*"/>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"##;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let _ = std::fs::write(out.join("addons-forever.exe.manifest"), MANIFEST);
    println!("cargo:rerun-if-changed=../../assets/app.ico");
    let mut rc = "1 24 \"addons-forever.exe.manifest\"\n".to_string();
    // Real app icon for the .exe (taskbar/Explorer) — docs/ART.md, assets/app.ico.
    let ico = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/app.ico");
    if ico.exists() && std::fs::copy(&ico, out.join("app.ico")).is_ok() {
        rc.push_str("1 ICON \"app.ico\"\n");
    }
    std::fs::write(out.join("app.rc"), rc).expect("write rc");

    let obj = out.join("app-res.obj");
    if compile_rc_msvc(&out, &obj) {
        // msvc rc.exe produces a .res that link.exe accepts directly
    } else if !compile_rc_windres(&out, &obj) {
        println!("cargo:warning=no resource compiler (WINDRES/windres/rc) found; building without DPI manifest");
        return;
    }
    println!("cargo:rustc-link-arg-bins={}", obj.display());
}

fn compile_rc_windres(out: &Path, obj: &Path) -> bool {
    let target = std::env::var("TARGET").unwrap_or_default();
    let prefixed = target.replace("-pc-windows-gnu", "-w64-mingw32");
    let mut candidates: Vec<Vec<String>> = Vec::new();
    if let Ok(w) = std::env::var("WINDRES") {
        candidates.push(vec![w]);
    }
    candidates.push(vec![format!("{prefixed}-windres")]);
    candidates.push(vec!["x86_64-w64-mingw32-windres".into()]);
    candidates.push(vec!["windres".into()]);
    // `--preprocessor=cpp` first: cross windres defaults to `<target>-gcc` for
    // preprocessing, which a windres-only install does not have.
    for pre in [Some("cpp"), None] {
        for exe in &candidates {
            let mut cmd = Command::new(&exe[0]);
            if let Some(pp) = pre {
                cmd.arg(format!("--preprocessor={pp}"));
            }
            cmd.args(["--output-format=coff", "-i"]).arg(out.join("app.rc")).arg("-o").arg(obj);
            if matches!(cmd.current_dir(out).status(), Ok(s) if s.success()) {
                return true;
            }
        }
    }
    false
}

fn compile_rc_msvc(out: &Path, obj: &Path) -> bool {
    if !std::env::var("CC").map(|c| c.contains("msvc")).unwrap_or(false)
        && !std::env::var("TARGET").map(|t| t.contains("msvc")).unwrap_or(false)
    {
        return false;
    }
    Command::new("rc.exe")
        .args(["/nologo", "/fo"])
        .arg(obj)
        .arg(out.join("app.rc"))
        .current_dir(out)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
