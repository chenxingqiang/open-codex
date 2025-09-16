extern crate icodex_execpolicy;

use std::vec;

use icodex_execpolicy::Error;
use icodex_execpolicy::ExecCall;
use icodex_execpolicy::MatchedExec;
use icodex_execpolicy::MatchedFlag;
use icodex_execpolicy::Policy;
use icodex_execpolicy::PositionalArg;
use icodex_execpolicy::ValidExec;
use icodex_execpolicy::get_default_policy;

#[expect(clippy::expect_used)]
fn setup() -> Policy {
    get_default_policy().expect("failed to load default policy")
}

#[test]
fn test_pwd_no_args() {
    let policy = setup();
    let pwd = ExecCall::new("pwd", &[]);
    assert_eq!(
        Ok(MatchedExec::Match {
            exec: ValidExec {
                program: "pwd".into(),
                ..Default::default()
            }
        }),
        policy.check(&pwd)
    );
}

#[test]
fn test_pwd_capital_l() {
    let policy = setup();
    let pwd = ExecCall::new("pwd", &["-L"]);
    assert_eq!(
        Ok(MatchedExec::Match {
            exec: ValidExec {
                program: "pwd".into(),
                flags: vec![MatchedFlag::new("-L")],
                ..Default::default()
            }
        }),
        policy.check(&pwd)
    );
}

#[test]
fn test_pwd_capital_p() {
    let policy = setup();
    let pwd = ExecCall::new("pwd", &["-P"]);
    assert_eq!(
        Ok(MatchedExec::Match {
            exec: ValidExec {
                program: "pwd".into(),
                flags: vec![MatchedFlag::new("-P")],
                ..Default::default()
            }
        }),
        policy.check(&pwd)
    );
}

#[test]
fn test_pwd_extra_args() {
    let policy = setup();
    let pwd = ExecCall::new("pwd", &["foo", "bar"]);
    assert_eq!(
        Err(Error::UnexpectedArguments {
            program: "pwd".to_string(),
            args: vec![
                PositionalArg {
                    index: 0,
                    value: "foo".to_string()
                },
                PositionalArg {
                    index: 1,
                    value: "bar".to_string()
                },
            ],
        }),
        policy.check(&pwd)
    );
}
