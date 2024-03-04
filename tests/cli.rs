use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn bad_directory() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/notexist");
    rupes.assert().failure();

    Ok(())
}

#[test]
fn empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/empty").arg("-S");
    rupes
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No files to scan, rupes will now exit",
        ));

    Ok(())
}

#[test]
fn default_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-r");
    rupes.assert().success().stdout(predicate::str::contains(
        "\n./example_files/test/a-file.txt\n./example_files/test/b-file.specialTXT\n",
    ));

    Ok(())
}

#[test]
fn recursive_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-r");

    rupes.assert().success().stdout(predicate::str::contains(
        "\n./example_files/test/.dot-dir/file-in-dot-dir.txt\n./example_files/test/a-file.txt\n./example_files/test/b-file.specialTXT\n\n./example_files/test/a-dir/.dot-file\n./example_files/test/a-dir/c-file.txt\n./example_files/test/a-dir/d-file.txt\n",
    ))
        .stdout(predicate::str::contains("./example_files/test2/1-file.txt").not());

    Ok(())
}

#[test]
fn recursive_scan_md5() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-r5");

    rupes.assert().success().stdout(predicate::str::contains(
        "\n./example_files/test/.dot-dir/file-in-dot-dir.txt\n./example_files/test/a-file.txt\n./example_files/test/b-file.specialTXT\n\n./example_files/test/a-dir/.dot-file\n./example_files/test/a-dir/c-file.txt\n./example_files/test/a-dir/d-file.txt\n",
    ))
        .stdout(predicate::str::contains("./example_files/test2/1-file.txt").not());

    Ok(())
}

#[test]
fn dotless_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-er");

    rupes.assert().success().stdout(predicate::str::contains(
        "\n./example_files/test/a-file.txt\n./example_files/test/b-file.specialTXT\n\n./example_files/test/a-dir/c-file.txt\n./example_files/test/a-dir/d-file.txt\n",
    ));

    Ok(())
}

#[test]
fn filtered_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes
        .arg("./example_files/test")
        .arg("-f")
        .arg("^.+[.]txt$")
        .arg("-r");

    rupes
        .assert()
        .success()
        .stdout(predicate::str::contains("./example_files/test/b-file.specialTXT").not())
        .stdout(predicate::str::contains("./example_files/test/a-file.txt"));

    Ok(())
}

#[test]
fn symlinks_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-rl");

    rupes
        .assert()
        .success()
        .stdout(predicate::str::contains("./example_files/test/test2/1-file.txt"));

    Ok(())
}

#[test]
fn max_constrained_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-rM").arg("18");

    rupes
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "./example_files/test/.dot-dir/file-in-dot-dir.txt",
        ))
        .stdout(predicate::str::contains("./example_files/test/a-dir/.dot-file").not());

    Ok(())
}

#[test]
fn min_constrained_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-rm").arg("19");

    rupes
        .assert()
        .success()
        .stdout(predicate::str::contains("./example_files/test/.dot-dir/file-in-dot-dir.txt").not())
        .stdout(predicate::str::contains(
            "./example_files/test/a-dir/.dot-file",
        ));

    Ok(())
}

#[test]
fn comma_separated_scan() -> Result<(), Box<dyn std::error::Error>> {
    let mut rupes = Command::cargo_bin("rupes")?;

    rupes.arg("./example_files/test").arg("-1").arg(", ");

    rupes.assert().success().stdout(predicate::str::contains(
        "./example_files/test/a-file.txt, ./example_files/test/b-file.specialTXT",
    ));

    Ok(())
}
