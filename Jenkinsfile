pipeline {
  agent any
  stages {
    stage('Verify cargo installation') {
      steps {
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
