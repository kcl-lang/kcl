pub mod capi;
pub(crate) mod into;
#[cfg(not(target_arch = "wasm32"))]
pub mod jsonrpc;
pub mod service_impl;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod ty;
pub(crate) mod util;

pub use service_impl::KclServiceImpl;

/// Single source of truth for the KCL service method registry.
///
/// Every name advertised by `BuiltinService.ListMethod` must be dispatchable
/// through the native FFI table ([`capi::lookup_service_fn_ptr`]) and
/// registered on the JSON-RPC server, and vice versa. The order follows the
/// `KclService` RPC declaration order in `spec.proto` — methods that were
/// removed from the spec (`BuildProgram`, `ExecArtifact`, ...) are
/// intentionally absent — followed by the `BuiltinService` methods.
pub(crate) const SERVICE_METHODS: &[&str] = &[
    "KclService.Ping",
    "KclService.GetVersion",
    "KclService.ParseProgram",
    "KclService.ParseFile",
    "KclService.LoadPackage",
    "KclService.ListOptions",
    "KclService.ListVariables",
    "KclService.ExecProgram",
    "KclService.OverrideFile",
    "KclService.GetSchemaTypeMapping",
    "KclService.GetSchemaTypeMappingUnderPath",
    "KclService.FormatCode",
    "KclService.FormatPath",
    "KclService.LintPath",
    "KclService.ValidateCode",
    "KclService.LoadSettingsFiles",
    "KclService.Rename",
    "KclService.RenameCode",
    "KclService.Test",
    "KclService.UpdateDependencies",
    "BuiltinService.Ping",
    "BuiltinService.ListMethod",
];

#[cfg(test)]
mod tests {
    use super::SERVICE_METHODS;
    use crate::service::capi::lookup_service_fn_ptr;
    use crate::service::service_impl::KclServiceImpl;

    /// Every method advertised by `BuiltinService.ListMethod` must have a
    /// native FFI entry so callers can actually invoke it.
    #[test]
    fn advertised_methods_are_dispatchable() {
        for name in SERVICE_METHODS {
            assert!(
                lookup_service_fn_ptr(name).is_some(),
                "advertised method {name} is missing from the native dispatch table"
            );
        }
        assert!(lookup_service_fn_ptr("KclService.DoesNotExist").is_none());
    }

    /// `KclServiceImpl::list_method` must return exactly the registry.
    #[test]
    fn list_method_matches_registry() {
        let serv = KclServiceImpl::default();
        let result = serv.list_method(&Default::default()).unwrap();
        let expected: Vec<String> = SERVICE_METHODS
            .iter()
            .map(|name| name.to_string())
            .collect();
        assert_eq!(result.method_name_list, expected);
    }

    /// Methods removed from the spec must not creep back into the registry.
    #[test]
    fn registry_excludes_removed_methods() {
        for removed in [
            "KclService.BuildProgram",
            "KclService.ExecArtifact",
            "KclService.ListDepFiles",
            "KclService.GetSchemaType",
            "KclService.GetFullSchemaType",
        ] {
            assert!(
                !SERVICE_METHODS.contains(&removed),
                "registry still advertises removed method {removed}"
            );
        }
    }
}
