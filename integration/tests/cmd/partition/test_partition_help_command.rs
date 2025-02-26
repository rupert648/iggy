use crate::cmd::common::{help::TestHelpCmd, IggyCmdTest, USAGE_PREFIX};
use serial_test::parallel;

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["partition", "help"],
            format!(
                r#"partition operations

{USAGE_PREFIX} partition <COMMAND>

Commands:
  create  Create partitions for the specified topic ID
              and stream ID based on the given count.
  delete  Delete partitions for the specified topic ID
              and stream ID based on the given count.
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
"#,
            ),
        ))
        .await;
}
