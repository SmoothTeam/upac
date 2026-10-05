// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::TypeId;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use upac_types::CancelToken;
use upac_types::error::{ErrorDomain, ErrorKind};
use upac_types::progress::ProgressEvent;
use upac_types::traits::CommandState;

use upac_orchestrator::context::Context;
use upac_orchestrator::error::PipelineError;
use upac_orchestrator::pipeline::Step;
use upac_orchestrator::stage::Stage;
use upac_orchestrator::{OrchestratorRun, SequentialOrchestrator, stages};

type Log = Arc<Mutex<Vec<String>>>;

type ReportedEvent = (u32, Option<String>, u64, u64);

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestError {
    Pipeline(PipelineError),
    Stage(&'static str),
}

impl From<PipelineError> for TestError {
    fn from(error: PipelineError) -> Self {
        TestError::Pipeline(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestState {
    Validation,
    Stage(usize),
}

impl CommandState for TestState {
    const DOMAIN: ErrorDomain = ErrorDomain::Unknown;
    const VALIDATION: Self = TestState::Validation;

    fn as_u32(self) -> u32 {
        match self {
            TestState::Validation => 0,
            TestState::Stage(index) => index as u32 + 1,
        }
    }

    fn from_stage_index(index: usize) -> Self {
        TestState::Stage(index)
    }
}

struct Done;

struct FinishStage;

impl Stage<TestError> for FinishStage {
    fn provides(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Done>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        context.put(Done);

        Ok(())
    }
}

struct RecordingStage {
    label: &'static str,
    log: Log,
}

impl Stage<TestError> for RecordingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        self.log.lock().unwrap().push(format!("run:{}", self.label));

        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        self.log.lock().unwrap().push(format!("rollback:{}", self.label));

        Ok(())
    }
}

struct FailingStage {
    label: &'static str,
    log: Log,
}

impl Stage<TestError> for FailingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        self.log.lock().unwrap().push(format!("run:{}", self.label));

        Err(TestError::Stage(self.label))
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        self.log.lock().unwrap().push(format!("rollback:{}", self.label));

        Ok(())
    }
}

struct ItemRecordingStage {
    label: &'static str,
    log: Log,
}

impl Stage<TestError> for ItemRecordingStage {
    fn requires(&self) -> Vec<TypeId> {
        vec![TypeId::of::<u32>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        let item = context.get::<u32>()?;
        self.log.lock().unwrap().push(format!("{}:{item}", self.label));

        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        self.log.lock().unwrap().push(format!("rollback:{}", self.label));

        Ok(())
    }
}

struct FailOnItemStage {
    fail_on: u32,
    log: Log,
}

impl Stage<TestError> for FailOnItemStage {
    fn requires(&self) -> Vec<TypeId> {
        vec![TypeId::of::<u32>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        let item = *context.get::<u32>()?;

        if item == self.fail_on {
            return Err(TestError::Stage("fail-on-item"));
        }

        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        self.log.lock().unwrap().push("rollback:fail-on-item".to_owned());

        Ok(())
    }
}

struct PushingStage;

impl Stage<TestError> for PushingStage {
    fn requires(&self) -> Vec<TypeId> {
        vec![TypeId::of::<u32>()]
    }

    fn provides(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Vec<u64>>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        let item = *context.get::<u32>()?;
        context.push(u64::from(item) * 10);

        Ok(())
    }
}

struct WideItemRecordingStage {
    log: Log,
}

impl Stage<TestError> for WideItemRecordingStage {
    fn requires(&self) -> Vec<TypeId> {
        vec![TypeId::of::<u64>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        let item = context.get::<u64>()?;
        self.log.lock().unwrap().push(format!("wide:{item}"));

        Ok(())
    }
}

struct CancellingStage;

impl Stage<TestError> for CancellingStage {
    fn run(
        &self, _context: &mut Context, cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        cancel.cancel();

        Ok(())
    }
}

struct BrokenRollbackStage;

impl Stage<TestError> for BrokenRollbackStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        Err(ErrorKind::WriteFailed)
    }
}

struct Marker;

struct ProvidesMarkerStage;

impl Stage<TestError> for ProvidesMarkerStage {
    fn provides(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Marker>()]
    }

    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        context.put(Marker);

        Ok(())
    }
}

struct RequiresMarkerStage {
    log: Log,
}

impl Stage<TestError> for RequiresMarkerStage {
    fn requires(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Marker>()]
    }

    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        self.log.lock().unwrap().push("run:requires-marker".to_owned());

        Ok(())
    }
}

struct ReportingStage;

impl Stage<TestError> for ReportingStage {
    fn run(
        &self, _context: &mut Context, _cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), TestError> {
        progress(Some("first"), 1, 2);
        progress(Some("second"), 2, 2);

        Ok(())
    }
}

fn new_log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

fn logged(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn run_reporting(steps: Vec<Step<TestError>>, context: &mut Context) -> Vec<ReportedEvent> {
    let reported = RefCell::new(Vec::new());
    let on_progress = |event: &ProgressEvent| {
        reported.borrow_mut().push((
            event.stage,
            event.subject.map(str::to_owned),
            event.current,
            event.total,
        ));
    };

    let result =
        SequentialOrchestrator::new(steps).run_unmutated::<Done, TestState>(context, &CancelToken::new(), &on_progress);
    assert!(result.is_ok());

    reported.into_inner()
}

fn run(
    mut steps: Vec<Step<TestError>>, context: &mut Context, cancel: &CancelToken,
) -> Result<Done, (TestState, TestError)> {
    steps.push(Step::once(Box::new(FinishStage)));

    SequentialOrchestrator::new(steps).run_unmutated(context, cancel, &|_| {})
}

#[test]
fn once_stages_run_in_declaration_order() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        RecordingStage {
            label: "b",
            log: Arc::clone(&log)
        },
    ];

    assert!(run(steps, &mut Context::default(), &CancelToken::new()).is_ok());
    assert_eq!(logged(&log), vec!["run:a", "run:b"]);
}

#[test]
fn a_failure_rolls_back_every_started_stage_in_reverse_order_including_the_failing_one() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        FailingStage {
            label: "fail",
            log: Arc::clone(&log)
        },
        RecordingStage {
            label: "c",
            log: Arc::clone(&log)
        },
    ];

    let result = run(steps, &mut Context::default(), &CancelToken::new());

    assert_eq!(result.err(), Some((TestState::Stage(1), TestError::Stage("fail"))));
    assert_eq!(logged(&log), vec!["run:a", "run:fail", "rollback:fail", "rollback:a"]);
}

#[test]
fn cancelling_before_the_first_stage_runs_nothing() {
    let log = new_log();
    let cancel = CancelToken::new();
    cancel.cancel();

    let steps = stages![RecordingStage {
        label: "a",
        log: Arc::clone(&log)
    }];

    let result = run(steps, &mut Context::default(), &cancel);

    assert_eq!(
        result.err(),
        Some((TestState::Stage(0), TestError::Pipeline(PipelineError::Cancelled)))
    );
    assert!(logged(&log).is_empty());
}

#[test]
fn cancelling_between_stages_stops_before_the_next_stage_and_rolls_back() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        CancellingStage,
        RecordingStage {
            label: "c",
            log: Arc::clone(&log)
        },
    ];

    let result = run(steps, &mut Context::default(), &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((TestState::Stage(2), TestError::Pipeline(PipelineError::Cancelled)))
    );
    assert_eq!(logged(&log), vec!["run:a", "rollback:a"]);
}

#[test]
fn a_failed_rollback_replaces_the_stage_error_and_still_unwinds_the_rest() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        BrokenRollbackStage,
        FailingStage {
            label: "fail",
            log: Arc::clone(&log)
        },
    ];

    let result = run(steps, &mut Context::default(), &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((TestState::Stage(2), TestError::Pipeline(PipelineError::RollbackFailed)))
    );
    assert_eq!(logged(&log), vec!["run:a", "run:fail", "rollback:fail", "rollback:a"]);
}

#[test]
fn each_runs_its_body_for_every_item_in_order() {
    let log = new_log();
    let mut context = Context::default();
    context.put(vec![1u32, 2]);

    let steps = stages![each::<u32>(
        ItemRecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        ItemRecordingStage {
            label: "b",
            log: Arc::clone(&log)
        },
    )];

    assert!(run(steps, &mut context, &CancelToken::new()).is_ok());
    assert_eq!(logged(&log), vec!["a:1", "b:1", "a:2", "b:2"]);
}

#[test]
fn each_takes_the_list_and_leaves_no_item_behind() {
    let mut context = Context::default();
    context.put(vec![1u32, 2]);

    let steps = stages![each::<u32>(ItemRecordingStage {
        label: "a",
        log: new_log()
    })];

    assert!(run(steps, &mut context, &CancelToken::new()).is_ok());
    assert!(context.get::<Vec<u32>>().is_err());
    assert!(context.get::<u32>().is_err());
}

#[test]
fn each_over_an_empty_list_runs_nothing() {
    let log = new_log();
    let mut context = Context::default();
    context.put(Vec::<u32>::new());

    let steps = stages![each::<u32>(ItemRecordingStage {
        label: "a",
        log: Arc::clone(&log)
    })];

    assert!(run(steps, &mut context, &CancelToken::new()).is_ok());
    assert!(logged(&log).is_empty());
}

#[test]
fn stages_inside_each_keep_flat_indices() {
    let log = new_log();
    let mut context = Context::default();
    context.put(vec![1u32, 2]);

    let steps = stages![
        RecordingStage {
            label: "x",
            log: Arc::clone(&log)
        },
        each::<u32>(
            ItemRecordingStage {
                label: "a",
                log: Arc::clone(&log)
            },
            FailOnItemStage {
                fail_on: 2,
                log: Arc::clone(&log)
            },
        ),
    ];

    let result = run(steps, &mut context, &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((TestState::Stage(2), TestError::Stage("fail-on-item")))
    );
}

#[test]
fn each_rolls_back_every_started_body_stage_once() {
    let log = new_log();
    let mut context = Context::default();
    context.put(vec![1u32, 2, 3]);

    let steps = stages![each::<u32>(
        ItemRecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        FailOnItemStage {
            fail_on: 2,
            log: Arc::clone(&log)
        },
    )];

    assert!(run(steps, &mut context, &CancelToken::new()).is_err());
    assert_eq!(logged(&log), vec!["a:1", "a:2", "rollback:fail-on-item", "rollback:a"]);
}

#[test]
fn pushed_results_feed_the_next_each() {
    let log = new_log();
    let mut context = Context::default();
    context.put(vec![1u32, 2]);

    let steps = stages![
        each::<u32>(PushingStage),
        each::<u64>(WideItemRecordingStage { log: Arc::clone(&log) }),
    ];

    assert!(run(steps, &mut context, &CancelToken::new()).is_ok());
    assert_eq!(logged(&log), vec!["wide:10", "wide:20"]);
}

#[test]
fn each_without_its_list_is_rejected_before_anything_runs() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        each::<u32>(ItemRecordingStage {
            label: "b",
            log: Arc::clone(&log)
        }),
    ];

    let result = run(steps, &mut Context::default(), &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((
            TestState::Validation,
            TestError::Pipeline(PipelineError::PipelineInvalid)
        ))
    );
    assert!(logged(&log).is_empty());
}

#[test]
fn the_item_is_not_available_after_its_each() {
    let mut context = Context::default();
    context.put(vec![1u32]);

    let steps = stages![
        each::<u32>(ItemRecordingStage {
            label: "a",
            log: new_log()
        }),
        ItemRecordingStage {
            label: "b",
            log: new_log()
        },
    ];

    let result = run(steps, &mut context, &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((
            TestState::Validation,
            TestError::Pipeline(PipelineError::PipelineInvalid)
        ))
    );
}

#[test]
fn a_requirement_nobody_provides_is_rejected_before_anything_runs() {
    let log = new_log();

    let steps = stages![
        RecordingStage {
            label: "a",
            log: Arc::clone(&log)
        },
        RequiresMarkerStage { log: Arc::clone(&log) },
    ];

    let result = run(steps, &mut Context::default(), &CancelToken::new());

    assert_eq!(
        result.err(),
        Some((
            TestState::Validation,
            TestError::Pipeline(PipelineError::PipelineInvalid)
        ))
    );
    assert!(logged(&log).is_empty());
}

#[test]
fn a_requirement_provided_by_an_earlier_stage_passes() {
    let log = new_log();

    let steps = stages![ProvidesMarkerStage, RequiresMarkerStage { log: Arc::clone(&log) }];

    assert!(run(steps, &mut Context::default(), &CancelToken::new()).is_ok());
    assert_eq!(logged(&log), vec!["run:requires-marker"]);
}

#[test]
fn a_requirement_already_in_the_context_passes() {
    let log = new_log();
    let mut context = Context::default();
    context.put(Marker);

    let steps = stages![RequiresMarkerStage { log: Arc::clone(&log) }];

    assert!(run(steps, &mut context, &CancelToken::new()).is_ok());
}

#[test]
fn run_unmutated_rejects_a_pipeline_that_never_provides_the_result() {
    let log = new_log();
    let orchestrator = SequentialOrchestrator::new(stages![RecordingStage {
        label: "a",
        log: Arc::clone(&log)
    }]);

    let result = orchestrator.run_unmutated::<Done, TestState>(&mut Context::default(), &CancelToken::new(), &|_| {});

    assert_eq!(
        result.err(),
        Some((
            TestState::Validation,
            TestError::Pipeline(PipelineError::PipelineInvalid)
        ))
    );
    assert!(logged(&log).is_empty());
}

#[test]
fn context_put_get_take_round_trip() {
    let mut context = Context::default();
    context.put(42u32);

    assert_eq!(context.get::<u32>(), Ok(&42));
    assert_eq!(context.take::<u32>(), Ok(42));
    assert_eq!(context.get::<u32>(), Err(PipelineError::MissingResult));
}

#[test]
fn context_push_collects_values_into_a_list() {
    let mut context = Context::default();
    context.push(1u32);
    context.push(2u32);

    assert_eq!(context.get::<Vec<u32>>(), Ok(&vec![1, 2]));
}

#[test]
fn replace_returns_the_value_it_overwrites() {
    let mut context = Context::default();
    context.put(1u32);

    assert_eq!(context.replace(2u32), Some(1));
    assert_eq!(context.get::<u32>(), Ok(&2));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "was already filled")]
fn put_into_an_occupied_slot_panics_in_debug_builds() {
    let mut context = Context::default();
    context.put(1u32);
    context.put(2u32);
}

#[test]
fn a_stage_reports_its_own_progress_under_its_index() {
    let reported = run_reporting(stages![ReportingStage, FinishStage], &mut Context::default());

    assert_eq!(
        reported,
        vec![
            (0, None, 0, 0),
            (0, Some("first".to_owned()), 1, 2),
            (0, Some("second".to_owned()), 2, 2),
            (1, None, 0, 0),
        ]
    );
}

#[test]
fn each_reports_the_item_position_around_every_body_stage() {
    let log = new_log();
    let mut context = Context::default();
    context.put(vec![7u32, 8u32]);

    let reported = run_reporting(
        stages![
            each::<u32>(ItemRecordingStage {
                label: "item",
                log: Arc::clone(&log)
            }),
            FinishStage,
        ],
        &mut context,
    );

    assert_eq!(
        reported,
        vec![
            (0, None, 0, 2),
            (0, None, 1, 2),
            (0, None, 1, 2),
            (0, None, 2, 2),
            (1, None, 0, 0),
        ]
    );
}
