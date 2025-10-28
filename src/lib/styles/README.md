# 🎨 Kuboard Color Palette Guide

This directory contains the centralized color management system for Kuboard. All colors are defined in `color-palette.css` for easy customization.

## 📁 Files

- **`color-palette.css`** - Main color definitions and theme variations
- **`variables.css`** - Spacing, sizing, and other non-color variables
- **`README.md`** - This guide (you are here!)

## 🆕 Recent Updates

### Resource Management Tabs
The color system now supports the new tabbed resource management interface:
- **Workloads Tab**: Pods, Deployments, Services
- **Nodes Tab**: Cluster nodes with detailed management
- **Config Tab**: ConfigMaps and Secrets
- **Network Tab**: Services and networking resources
- **Custom Resources Tab**: CRDs and custom resources

### Enhanced Status Badges
Improved visibility and contrast for status indicators:
- **Ready/Running/Available**: Darker green with better contrast
- **Pending/Not Ready**: Amber warning colors
- **Error/Failed**: Red error colors
- **Info/ClusterIP**: Blue info colors

## 🚀 Quick Start

### In Development Mode

1. **Start the dev server:**
   ```bash
   npm run dev
   ```

2. **Use the Theme Switcher:**
   - Look for the 🎨 button in the top-right corner
   - Click it to open the theme panel
   - Switch between Dark, Light, and High Contrast themes
   - Click "📝 Edit Colors" to open `color-palette.css` in your editor

3. **Edit Colors:**
   - Modify colors in `src/lib/styles/color-palette.css`
   - Save the file
   - Click "🔄 Reload" in the theme switcher to see changes

### Color Categories

#### 🎯 Primary Brand Colors
```css
--primary-color: #3b82f6;      /* Main brand color */
--primary-dark: #1d4ed8;       /* Darker shade */
--primary-light: #60a5fa;      /* Lighter shade */
```

#### ✅ Status Colors
```css
--success-color: #10b981;      /* Green for success/ready */
--warning-color: #f59e0b;      /* Amber for warnings */
--error-color: #ef4444;        /* Red for errors */
--info-color: #06b6d4;         /* Cyan for info */
```

#### 🌑 Background Colors
```css
--background-primary: #000000;   /* Main background */
--background-secondary: #111111; /* Secondary background */
--background-card: #1a1a1a;      /* Card backgrounds */
```

#### 📝 Text Colors
```css
--text-primary: #ffffff;        /* Primary text */
--text-secondary: #a1a1aa;      /* Secondary text */
--text-muted: #71717a;          /* Muted text */
```

#### 🏷️ Status Badge Colors
```css
--status-ready-bg: rgba(16, 185, 129, 0.4);  /* Ready state background */
--status-ready-text: #059669;                /* Ready state text */
--status-ready-border: rgba(16, 185, 129, 0.6); /* Ready state border */
```

## 🎨 Customization Examples

### Change Primary Color to Purple
```css
--primary-color: #8b5cf6;
--primary-dark: #7c3aed;
--primary-light: #a78bfa;
```

### Make Success Color Brighter
```css
--success-color: #22c55e;
--success-dark: #16a34a;
--status-ready-text: #16a34a;
```

### Adjust Status Badge Opacity
```css
--status-ready-bg: rgba(16, 185, 129, 0.6);  /* More opaque */
--status-ready-border: rgba(16, 185, 129, 0.8); /* Stronger border */
```

### Change Background to Darker
```css
--background-primary: #0a0a0a;
--background-secondary: #0f0f0f;
--background-card: #141414;
```

## 🔧 Advanced Customization

### Create a New Theme
Add a new theme variation in `color-palette.css`:

```css
[data-theme="custom"] {
  --primary-color: #your-color;
  --background-primary: #your-bg;
  /* ... other overrides */
}
```

### Add New Color Variables
1. Add the variable to the `:root` section
2. Use it in your components
3. Document it in this README

### Component-Specific Colors
For colors used in only one component, define them locally in that component's `<style>` section.

## 📋 Best Practices

1. **Use Semantic Names:** `--success-color` instead of `--green`
2. **Maintain Consistency:** Use the same color for similar purposes
3. **Test Accessibility:** Ensure sufficient contrast ratios
4. **Document Changes:** Update this README when adding new colors
5. **Use CSS Variables:** Always use `var(--color-name)` instead of hardcoded values

## 🎯 Common Use Cases

### Making Text More Visible
```css
--text-primary: #ffffff;        /* Pure white for maximum contrast */
--text-secondary: #e5e7eb;      /* Brighter secondary text */
```

### Softening Colors
```css
--primary-color: #6366f1;       /* Softer blue */
--success-color: #059669;       /* Darker, more muted green */
```

### High Contrast Mode
```css
--background-primary: #000000;
--text-primary: #ffffff;
--primary-color: #00ff00;       /* Bright green for visibility */
```

## 🔍 Troubleshooting

### Colors Not Updating
1. Check browser cache (Ctrl+F5)
2. Verify CSS syntax
3. Ensure variables are defined in `:root`
4. Check for typos in variable names

### Theme Switcher Not Working
1. Make sure you're in dev mode (`npm run dev`)
2. Check browser console for errors
3. Verify `ThemeSwitcher.svelte` is imported

### Inconsistent Colors
1. Check if components are using hardcoded colors
2. Verify all components import `color-palette.css`
3. Look for CSS specificity issues

## 📚 Related Files

- `src/lib/components/ThemeSwitcher.svelte` - Dev mode theme switcher
- `src/lib/styles/variables.css` - Non-color variables
- `src/app.html` - Global styles and theme initialization

---

**Happy Theming! 🎨✨**
