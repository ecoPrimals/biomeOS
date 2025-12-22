//! Minimal ISO Creator Mock Data

/// Minimal advanced settings
pub struct AdvancedSettings {
    pub custom_kernel: bool,
    pub boot_timeout: u32,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            custom_kernel: false,
            boot_timeout: 30,
        }
    }
}

/// Mock provider for ISO creator functionality
pub struct IsoCreatorMockProvider;

/// Global mock provider instance
pub static ISO_CREATOR_MOCK_PROVIDER: IsoCreatorMockProvider = IsoCreatorMockProvider;

impl IsoCreatorMockProvider {
    pub fn get_advanced_settings(&self) -> AdvancedSettings {
        AdvancedSettings::default()
    }
} 