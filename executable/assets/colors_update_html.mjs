// Function to update the meta theme-color from CSS variable
const updateThemeColor = () => {
    // Get the CSS variable value
    const themeColor = getComputedStyle(document.documentElement)
        .getPropertyValue('--theme-color').trim();
    
    // Update the meta tag with the theme color
    const metaThemeColor = document.querySelector('meta[name="theme-color"]');
    if (metaThemeColor) {
        metaThemeColor.setAttribute('content', themeColor);
    }
}

// Wait until the document is fully loaded to set the theme color
document.addEventListener('DOMContentLoaded', updateThemeColor);
