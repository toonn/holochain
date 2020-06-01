use crate::core::ribosome::FnComponents;
use crate::core::ribosome::Invocation;
use crate::core::ribosome::ZomesToInvoke;
use crate::fixt::DnaDefFixturator;
use crate::fixt::MigrateAgentFixturator;
use fixt::prelude::*;
use holochain_serialized_bytes::prelude::*;
use holochain_types::dna::DnaDef;
use holochain_zome_types::migrate_agent::MigrateAgent;
use holochain_zome_types::migrate_agent::MigrateAgentCallbackResult;
use holochain_zome_types::zome::ZomeName;
use holochain_zome_types::HostInput;

#[derive(Clone)]
pub struct MigrateAgentInvocation {
    dna_def: DnaDef,
    migrate_agent: MigrateAgent,
}

impl MigrateAgentInvocation {
    pub fn new(dna_def: DnaDef, migrate_agent: MigrateAgent) -> Self {
        Self {
            dna_def,
            migrate_agent,
        }
    }
}

fixturator!(
    MigrateAgentInvocation;
    constructor fn new(DnaDef, MigrateAgent);
);

impl Invocation for MigrateAgentInvocation {
    fn allow_side_effects(&self) -> bool {
        false
    }
    fn zomes(&self) -> ZomesToInvoke {
        ZomesToInvoke::All
    }
    fn fn_components(&self) -> FnComponents {
        vec![
            "migrate_agent".into(),
            match self.migrate_agent {
                MigrateAgent::Open => "open",
                MigrateAgent::Close => "close",
            }
            .into(),
        ]
        .into()
    }
    fn host_input(self) -> Result<HostInput, SerializedBytesError> {
        Ok(HostInput::new((&self.migrate_agent).try_into()?))
    }
}

impl TryFrom<MigrateAgentInvocation> for HostInput {
    type Error = SerializedBytesError;
    fn try_from(migrate_agent_invocation: MigrateAgentInvocation) -> Result<Self, Self::Error> {
        Ok(Self::new(
            (&migrate_agent_invocation.migrate_agent).try_into()?,
        ))
    }
}

/// the aggregate result of all zome callbacks for migrating an agent between dnas
#[derive(PartialEq, Debug)]
pub enum MigrateAgentResult {
    /// all implemented migrate agent callbacks in all zomes passed
    Pass,
    /// some migrate agent callback failed
    /// ZomeName is the first zome that failed
    /// String is some human readable string explaining the failure
    Fail(ZomeName, String),
}

impl From<Vec<MigrateAgentCallbackResult>> for MigrateAgentResult {
    fn from(callback_results: Vec<MigrateAgentCallbackResult>) -> Self {
        callback_results.into_iter().fold(Self::Pass, |acc, x| {
            match x {
                // fail always overrides the acc
                MigrateAgentCallbackResult::Fail(zome_name, fail_string) => {
                    Self::Fail(zome_name, fail_string)
                }
                // pass allows the acc to continue
                MigrateAgentCallbackResult::Pass => acc,
            }
        })
    }
}

#[cfg(test)]
mod test {

    use super::MigrateAgentInvocationFixturator;
    use super::MigrateAgentResult;
    use crate::core::ribosome::Invocation;
    use crate::core::ribosome::RibosomeT;
    use crate::core::ribosome::ZomesToInvoke;
    use crate::core::workflow::unsafe_invoke_zome_workspace::UnsafeInvokeZomeWorkspaceFixturator;
    use crate::fixt::curve::Zomes;
    use crate::fixt::MigrateAgentFixturator;
    use crate::fixt::WasmRibosomeFixturator;
    use crate::fixt::ZomeNameFixturator;
    use holochain_serialized_bytes::prelude::*;
    use holochain_wasm_test_utils::TestWasm;
    use holochain_zome_types::migrate_agent::MigrateAgent;
    use holochain_zome_types::migrate_agent::MigrateAgentCallbackResult;
    use holochain_zome_types::HostInput;
    use rand::prelude::*;

    #[tokio::test(threaded_scheduler)]
    async fn migrate_agent_callback_result_fold() {
        let mut rng = thread_rng();

        let result_pass = || MigrateAgentResult::Pass;
        let result_fail = || {
            MigrateAgentResult::Fail(
                ZomeNameFixturator::new(fixt::Empty).next().unwrap(),
                "".into(),
            )
        };

        let cb_pass = || MigrateAgentCallbackResult::Pass;
        let cb_fail = || {
            MigrateAgentCallbackResult::Fail(
                ZomeNameFixturator::new(fixt::Empty).next().unwrap(),
                "".into(),
            )
        };

        for (mut results, expected) in vec![
            (vec![], result_pass()),
            (vec![cb_pass()], result_pass()),
            (vec![cb_fail()], result_fail()),
            (vec![cb_fail(), cb_pass()], result_fail()),
        ] {
            // order of the results should not change the final result
            results.shuffle(&mut rng);

            // number of times a callback result appears should not change the final result
            let number_of_extras = rng.gen_range(0, 5);
            for _ in 0..number_of_extras {
                let maybe_extra = results.choose(&mut rng).cloned();
                match maybe_extra {
                    Some(extra) => results.push(extra),
                    _ => {}
                };
            }

            assert_eq!(expected, results.into(),);
        }
    }

    #[tokio::test(threaded_scheduler)]
    async fn migrate_agent_invocation_allow_side_effects() {
        let migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        assert!(!migrate_agent_invocation.allow_side_effects());
    }

    #[tokio::test(threaded_scheduler)]
    async fn migrate_agent_invocation_zomes() {
        let migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        assert_eq!(ZomesToInvoke::All, migrate_agent_invocation.zomes(),);
    }

    #[tokio::test(threaded_scheduler)]
    async fn migrate_agent_invocation_fn_components() {
        let mut migrate_agent_invocation =
            MigrateAgentInvocationFixturator::new(fixt::Unpredictable)
                .next()
                .unwrap();

        migrate_agent_invocation.migrate_agent = MigrateAgent::Open;

        let mut expected = vec!["migrate_agent", "migrate_agent_open"];
        for fn_component in migrate_agent_invocation.fn_components() {
            assert_eq!(fn_component, expected.pop().unwrap());
        }
    }

    #[tokio::test(threaded_scheduler)]
    async fn migrate_agent_invocation_host_input() {
        let migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Empty)
            .next()
            .unwrap();

        let host_input = migrate_agent_invocation.clone().host_input().unwrap();

        assert_eq!(
            host_input,
            HostInput::new(
                SerializedBytes::try_from(MigrateAgentFixturator::new(fixt::Empty).next().unwrap())
                    .unwrap()
            ),
        );
    }

    #[tokio::test(threaded_scheduler)]
    #[serial_test::serial]
    async fn test_migrate_agent_unimplemented() {
        let workspace = UnsafeInvokeZomeWorkspaceFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        let ribosome = WasmRibosomeFixturator::new(Zomes(vec![TestWasm::Foo]))
            .next()
            .unwrap();
        let mut migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Empty)
            .next()
            .unwrap();
        migrate_agent_invocation.dna_def = ribosome.dna_file.dna.clone();

        let result = ribosome
            .run_migrate_agent(workspace, migrate_agent_invocation)
            .unwrap();
        assert_eq!(result, MigrateAgentResult::Pass,);
    }

    #[tokio::test(threaded_scheduler)]
    #[serial_test::serial]
    async fn test_migrate_agent_implemented_pass() {
        let workspace = UnsafeInvokeZomeWorkspaceFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        let ribosome = WasmRibosomeFixturator::new(Zomes(vec![TestWasm::MigrateAgentPass]))
            .next()
            .unwrap();
        let mut migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Empty)
            .next()
            .unwrap();
        migrate_agent_invocation.dna_def = ribosome.dna_file.dna.clone();

        let result = ribosome
            .run_migrate_agent(workspace, migrate_agent_invocation)
            .unwrap();
        assert_eq!(result, MigrateAgentResult::Pass,);
    }

    #[tokio::test(threaded_scheduler)]
    #[serial_test::serial]
    async fn test_migrate_agent_implemented_fail() {
        let workspace = UnsafeInvokeZomeWorkspaceFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        let ribosome = WasmRibosomeFixturator::new(Zomes(vec![TestWasm::MigrateAgentFail]))
            .next()
            .unwrap();
        let mut migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Empty)
            .next()
            .unwrap();
        migrate_agent_invocation.dna_def = ribosome.dna_file.dna.clone();

        let result = ribosome
            .run_migrate_agent(workspace, migrate_agent_invocation)
            .unwrap();
        assert_eq!(
            result,
            MigrateAgentResult::Fail(TestWasm::MigrateAgentFail.into(), "no migrate".into()),
        );
    }

    #[tokio::test(threaded_scheduler)]
    #[serial_test::serial]
    async fn test_migrate_agent_multi_implemented_fail() {
        let workspace = UnsafeInvokeZomeWorkspaceFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        let ribosome = WasmRibosomeFixturator::new(Zomes(vec![
            TestWasm::MigrateAgentPass,
            TestWasm::MigrateAgentFail,
        ]))
        .next()
        .unwrap();
        let mut migrate_agent_invocation = MigrateAgentInvocationFixturator::new(fixt::Empty)
            .next()
            .unwrap();
        migrate_agent_invocation.dna_def = ribosome.dna_file.dna.clone();

        let result = ribosome
            .run_migrate_agent(workspace, migrate_agent_invocation)
            .unwrap();
        assert_eq!(
            result,
            MigrateAgentResult::Fail(TestWasm::MigrateAgentFail.into(), "no migrate".into()),
        );
    }
}