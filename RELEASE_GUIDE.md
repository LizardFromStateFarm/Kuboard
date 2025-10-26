# 🚀 Kuboard Release Guide

This guide explains how to create releases and distribute executables for Kuboard.

## 📦 **Automated Release Process**

### **Creating a Release**

1. **Create a Version Tag:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions Will Automatically:**
   - Build executables for Windows, macOS (Intel + Apple Silicon), and Linux
   - Create a GitHub release with all download links
   - Upload all platform-specific installers

### **Release Files Generated:**

**Windows:**
- `Kuboard_v1.0.0_x64-setup.exe` (NSIS installer)
- `Kuboard_v1.0.0_x64_en-US.msi` (MSI installer)

**macOS:**
- `Kuboard_v1.0.0_x64.dmg` (Intel Macs)
- `Kuboard_v1.0.0_aarch64.dmg` (Apple Silicon)

**Linux:**
- `Kuboard_v1.0.0_x64.AppImage` (Portable executable)

## 🛠️ **Manual Release Process**

### **Prerequisites for Manual Builds:**

**Windows:**
- Visual Studio Build Tools
- Windows SDK
- Rust toolchain

**macOS:**
- Xcode Command Line Tools
- Rust toolchain
- Code signing certificates (for distribution)

**Linux:**
- GTK development libraries
- WebKit2GTK development libraries
- Rust toolchain

### **Manual Build Commands:**

```bash
# Install dependencies
npm install

# Build frontend
npm run build

# Build Tauri app
npm run tauri build
```

### **Build Outputs:**

The build process creates installers in:
- `src-tauri/target/release/bundle/msi/` - Windows MSI
- `src-tauri/target/release/bundle/nsis/` - Windows NSIS
- `src-tauri/target/release/bundle/dmg/` - macOS DMG
- `src-tauri/target/release/bundle/appimage/` - Linux AppImage

## 🏷️ **Version Management**

### **Semantic Versioning:**
- **Major** (v2.0.0): Breaking changes
- **Minor** (v1.1.0): New features, backward compatible
- **Patch** (v1.0.1): Bug fixes, backward compatible

### **Pre-release Versions:**
- **Alpha** (v1.0.0-alpha.1): Early development
- **Beta** (v1.0.0-beta.1): Feature complete, testing
- **Release Candidate** (v1.0.0-rc.1): Final testing

## 📋 **Release Checklist**

### **Before Creating a Release:**

- [ ] **Code Quality:**
  - [ ] All tests pass
  - [ ] No TypeScript/Rust compilation errors
  - [ ] Code review completed
  - [ ] Documentation updated

- [ ] **Version Updates:**
  - [ ] Update version in `package.json`
  - [ ] Update version in `src-tauri/Cargo.toml`
  - [ ] Update version in `src-tauri/tauri.conf.json`
  - [ ] Update CHANGELOG.md

- [ ] **Testing:**
  - [ ] Test on target platforms
  - [ ] Verify all features work
  - [ ] Test installation process
  - [ ] Verify uninstallation works

### **Creating the Release:**

1. **Create and Push Tag:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Monitor GitHub Actions:**
   - Go to Actions tab in GitHub
   - Watch the build progress
   - Check for any build failures

3. **Review Release:**
   - Go to Releases page
   - Review generated release notes
   - Edit release description if needed
   - Publish the release

## 🔧 **Troubleshooting Builds**

### **Common Issues:**

**Windows Build Fails:**
- Ensure Visual Studio Build Tools are installed
- Check Windows SDK installation
- Verify Rust toolchain is up to date

**macOS Build Fails:**
- Install Xcode Command Line Tools: `xcode-select --install`
- Check code signing certificates
- Verify Apple Developer account access

**Linux Build Fails:**
- Install required libraries:
  ```bash
  sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
  ```

### **Build Artifacts:**

If builds fail, check:
- GitHub Actions logs for specific errors
- Local build environment setup
- Dependencies and toolchain versions
- Platform-specific requirements

## 📊 **Release Analytics**

### **Download Tracking:**
- GitHub provides download statistics
- Monitor which platforms are most popular
- Track release adoption rates

### **User Feedback:**
- Monitor GitHub Issues for release feedback
- Track user-reported bugs
- Collect feature requests

## 🎯 **Best Practices**

1. **Regular Releases:** Aim for monthly releases
2. **Stable Releases:** Test thoroughly before release
3. **Clear Communication:** Write comprehensive release notes
4. **User Support:** Monitor and respond to user issues
5. **Security Updates:** Release security patches promptly

---

**Happy Releasing!** 🚀
