# 🤝 Contributing to Kuboard

Thank you for your interest in contributing to Kuboard! This guide will help you get started.

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Tauri CLI (`cargo install tauri-cli`)
- Git

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/your-username/kuboard.git
cd kuboard

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

## 📋 Contribution Guidelines

### 1. **Code Style**

**Frontend (Svelte/TypeScript):**
- Use TypeScript for all components
- Follow Svelte best practices
- Use CSS custom properties from `src/lib/styles/variables.css`
- Include proper ARIA roles for accessibility

**Backend (Rust):**
- Use `kuboard_` prefix for all functions
- Follow Rust naming conventions
- Include comprehensive error handling
- Add documentation comments

### 2. **Pull Request Process**

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Test** thoroughly
5. **Commit** with clear messages (`git commit -m 'Add amazing feature'`)
6. **Push** to your branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

### 3. **Commit Messages**

Use clear, descriptive commit messages:
```
feat: add node metrics visualization
fix: resolve context switching issue
docs: update component documentation
style: improve button hover effects
refactor: extract utility functions
```

### 4. **Testing**

Before submitting:
- [ ] Test on your target platform (Windows/macOS/Linux)
- [ ] Verify all functionality works
- [ ] Check for TypeScript/Rust compilation errors
- [ ] Test with different Kubernetes contexts
- [ ] Verify UI responsiveness

## 🎯 Areas for Contribution

### **Frontend**
- UI/UX improvements
- New component features
- Accessibility enhancements
- Performance optimizations
- Mobile responsiveness

### **Backend**
- New Kubernetes resource support
- Metrics server integration
- Performance improvements
- Error handling enhancements
- API optimizations

### **Documentation**
- Code documentation
- User guides
- API references
- Tutorial content

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Platform** (Windows/macOS/Linux)
2. **Kuboard version**
3. **Steps to reproduce**
4. **Expected behavior**
5. **Actual behavior**
6. **Screenshots** (if applicable)

## 💡 Feature Requests

For new features:

1. **Check** existing issues first
2. **Describe** the feature clearly
3. **Explain** the use case
4. **Consider** implementation complexity
5. **Discuss** in GitHub Discussions

## 📚 Development Resources

- **[Architecture Overview](ARCHITECTURE_OVERVIEW.md)** - System architecture
- **[UI Organization](UI_ORGANIZATION_GUIDE.md)** - Component structure
- **[Function Documentation](FUNCTION_DOCUMENTATION.md)** - API reference
- **[Build Guide](BUILD_GUIDE.md)** - Build instructions

## 🏷️ Labels

We use labels to categorize issues:
- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Improvements to documentation
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed

## 📞 Getting Help

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - Questions and general discussion
- **Pull Requests** - Code contributions

## 🎉 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to Kuboard! 🚢
