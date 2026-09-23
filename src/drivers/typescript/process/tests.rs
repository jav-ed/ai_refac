use super::*;
use std::path::Path;

fn fixture(mode: &str, pid_file: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "drivers::typescript::process::tests::process_fixture",
            "--nocapture",
        ])
        .env("REFAC_PROCESS_TEST_MODE", mode)
        .env("REFAC_PROCESS_TEST_PID", pid_file);
    command
}

fn assert_reaped(pid_file: &Path) {
    let pid = std::fs::read_to_string(pid_file)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    assert!(
        system.process(Pid::from_u32(pid)).is_none(),
        "helper {pid} survived failure"
    );
}

#[test]
#[ignore = "subprocess fixture invoked only by the supervisor tests"]
fn process_fixture() {
    let Ok(mode) = std::env::var("REFAC_PROCESS_TEST_MODE") else {
        return;
    };
    std::fs::write(
        std::env::var("REFAC_PROCESS_TEST_PID").unwrap(),
        std::process::id().to_string(),
    )
    .unwrap();
    if mode == "success" {
        println!("helper stdout");
        eprintln!("helper stderr");
        return;
    }
    let allocation = if mode == "memory" {
        vec![42_u8; 64 * 1024 * 1024]
    } else {
        vec![]
    };
    loop {
        std::hint::black_box(&allocation);
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[tokio::test]
async fn timeout_terminates_and_reaps_helper() {
    let temp = tempfile::tempdir().unwrap();
    let pid_file = temp.path().join("pid");
    let error = run(
        &mut fixture("idle", &pid_file),
        Limits {
            timeout: Duration::from_millis(500),
            rss_bytes: 256 * MIB,
        },
    )
    .await
    .unwrap_err();
    assert!(format!("{error:#}").contains("timed out"));
    assert_reaped(&pid_file);
}

#[tokio::test]
async fn memory_limit_terminates_and_reaps_helper() {
    let temp = tempfile::tempdir().unwrap();
    let pid_file = temp.path().join("pid");
    let error = run(
        &mut fixture("memory", &pid_file),
        Limits {
            timeout: Duration::from_secs(5),
            rss_bytes: 32 * MIB,
        },
    )
    .await
    .unwrap_err();
    assert!(format!("{error:#}").contains("exceeded its RAM limit"));
    assert_reaped(&pid_file);
}

#[tokio::test]
async fn successful_helper_preserves_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = run(
        &mut fixture("success", &temp.path().join("pid")),
        Limits {
            timeout: Duration::from_secs(5),
            rss_bytes: 256 * MIB,
        },
    )
    .await
    .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("helper stdout"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("helper stderr"));
}

#[tokio::test]
async fn cancelling_supervisor_terminates_helper() {
    let temp = tempfile::tempdir().unwrap();
    let pid_file = temp.path().join("pid");
    let mut command = fixture("idle", &pid_file);
    let task = tokio::spawn(async move {
        run(
            &mut command,
            Limits {
                timeout: Duration::from_secs(30),
                rss_bytes: 256 * MIB,
            },
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        while !pid_file.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    // kill-on-drop reaping is asynchronous in Tokio; allow the runtime to reap.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_reaped(&pid_file);
}
