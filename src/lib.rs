use anyhow::Result;
use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::configuration::get_unknown_property_diagnostics;
use dprint_core::configuration::get_value;
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub line_width: usize,
    pub indent_width: isize,
}

#[derive(Default)]
pub struct CedarFormatter;

impl SyncPluginHandler<Configuration> for CedarFormatter {
    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "cedar".to_string(),
            help_url: "https://github.com/scufflecloud/dprint-plugin-cedar".to_string(),
            config_schema_url: String::new(),
            update_url: None,
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../LICENSE").to_string()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let mut config = config;
        let mut diagnostics = Vec::new();
        let line_width = get_value(
            &mut config,
            "lineWidth",
            global_config.line_width.unwrap_or(80) as usize,
            &mut diagnostics,
        );

        let indent_width = get_value(
            &mut config,
            "indentWidth",
            global_config.indent_width.unwrap_or(2) as isize,
            &mut diagnostics,
        );

        diagnostics.extend(get_unknown_property_diagnostics(config));

        PluginResolveConfigurationResult {
            config: Configuration { line_width, indent_width },
            diagnostics,
            file_matching: FileMatchingInfo {
                file_extensions: vec!["cedar".to_string()],
                file_names: vec![],
            },
        }
    }

    fn check_config_updates(
        &self,
        _message: CheckConfigUpdatesMessage,
    ) -> Result<Vec<ConfigChange>> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<Configuration>,
        _: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        let data = String::from_utf8(request.file_bytes).unwrap();
        let text = cedar_policy_formatter::policies_str_to_pretty(&data, &cedar_policy_formatter::Config {
            line_width: request.config.line_width,
            indent_width: request.config.indent_width,
        }).map_err(|e| {
            anyhow::anyhow!(e.with_source_code(data).to_string())
        })?;

        Ok(Some(text.into()))
    }
}

generate_plugin_code!(CedarFormatter, CedarFormatter);
