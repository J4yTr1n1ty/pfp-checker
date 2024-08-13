pipeline {
  agent any
  options {
    cache(path: 'target', key: 'rust-cache')
  }
  stages {
    stage('Verify cargo installation') {
      steps {
        sh '. "$HOME/.cargo/env"'
        sh 'cargo --version'
      }
    }
    stage('Install Database CLI') {
      steps {
        sh 'cargo install sqlx-cli'
      }
    }
    stage('Database Migration') {
      steps {
        sh 'sqlx database setup --database-url sqlite:database.sqlite'
      }
    }
    stage('Build') {
      steps {
        sh 'cargo build --verbose'
      }
    }
    stage('Test') {
      steps {
        sh 'cargo test --verbose'
      }
    }
  }
}
