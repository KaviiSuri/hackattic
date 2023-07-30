use base64::{engine::general_purpose, Engine as _};
use std::str::from_utf8;

use tempfile::{tempdir, TempDir};
use tokio::io::AsyncWriteExt;

pub struct DB {
    pub dir: TempDir,
    pub dump_b64: String,
    pub container_name: Option<String>,
}

const DOCKERFILE_CONTENT: &str = r#"
FROM postgres:10.8-alpine

ENV POSTGRES_DB testdb
ENV POSTGRES_USER postgres
ENV POSTGRES_PASSWORD postgres

COPY ./db.dump /usr/local/bin/db.dump
COPY ./init.sh /docker-entrypoint-initdb.d/init.sh
"#;

const INIT_SH_CONTENT: &str = r#"
#!/bin/bash

psql -U $POSTGRES_USER -d $POSTGRES_DB -a -f /usr/local/bin/db.dump
"#;

const DB_NAME: &str = "testdb";

impl DB {
    pub async fn new(dump_b64: String) -> Self {
        let dir = tempdir().expect("Could not create tempdir");
        let s = Self {
            dir,
            dump_b64,
            container_name: None,
        };
        s.setup().await;
        return s;
    }

    pub async fn build(&self) {
        let output = tokio::process::Command::new("docker")
            .current_dir(self.dir.path())
            .args(&["build", "--no-cache", "-t", DB_NAME, "."])
            .output()
            .await
            .expect("failed to build docker image");
        println!("build output: {:?}", output);
    }

    pub async fn run(&mut self) {
        let output = tokio::process::Command::new("docker")
            .current_dir(self.dir.path())
            .args(&["run", "-d", "-p", "5432:5432", DB_NAME])
            .output()
            .await
            .expect("failed to run docker image");
        // escape the newline character
        self.container_name = Some(from_utf8(&output.stdout).unwrap().trim().to_string());
        println!("run output: {:?}", output);

        // wait for the container to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    pub async fn get_psql_url(&self) -> String {
        return format!("postgres://postgres:postgres@localhost:5432/{DB_NAME}").to_string();
    }

    async fn setup(&self) {
        self.create_docker_file().await;
        self.create_init_file().await;
        self.create_dump_file().await;
        self.build().await;
    }
}

impl DB {
    async fn create_docker_file(&self) {
        let mut file = tokio::fs::File::create(self.dir.path().join("Dockerfile"))
            .await
            .expect("Could not create Dockerfile");

        file.write_all(DOCKERFILE_CONTENT.as_bytes())
            .await
            .expect("Could not write to Dockerfile");
    }

    async fn create_init_file(&self) {
        let mut file = tokio::fs::File::create(self.dir.path().join("init.sh"))
            .await
            .expect("Could not create Dockerfile");

        file.write_all(INIT_SH_CONTENT.as_bytes())
            .await
            .expect("Could not write to Dockerfile");
    }

    async fn create_dump_file(&self) {
        let file_content = general_purpose::STANDARD
            .decode(&self.dump_b64)
            .expect("Could not decode dump_b64");
        let mut file = tokio::fs::File::create(self.dir.path().join("db.dump.gz"))
            .await
            .expect("Could not create db.dump.gz");

        file.write_all(&file_content)
            .await
            .expect("Could not write to db.dump.gz");

        tokio::process::Command::new("gunzip")
            .current_dir(self.dir.path())
            .arg("db.dump.gz")
            .output()
            .await
            .expect("failed to gunzip dump file");
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        if let Some(container_name) = &self.container_name {
            std::process::Command::new("docker")
                .current_dir(self.dir.path())
                .arg("kill")
                .arg(container_name)
                .output()
                .expect("failed to run docker image");
        };
    }
}
