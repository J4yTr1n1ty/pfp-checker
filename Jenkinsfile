pipeline {
  agent any
  stages {
    stage('Verify cargo installation') {
      steps {
        sh '. "$HOME/.cargo/env"'
        sh 'cargo --version'
      }
    }
    stage('Install Database CLI') {
      steps {
        cache(maxCacheSize: 1, caches: [
          [path: 'target', key: 'rust-cache']
        ]) {
          sh 'cargo install sqlx-cli'
        }
      }
    }
    stage('Database Migration') {
      steps {
        sh 'sqlx database setup --database-url sqlite:database.sqlite'
      }
    }
    stage('Build') {
      steps {
        cache(maxCacheSize: 1, caches: [
          [path: 'target', key: 'rust-cache']
        ]) {
          sh 'cargo build --verbose'
        }
      }
    }
    stage('Test') {
      steps {
        cache(maxCacheSize: 1, caches: [
          [path: 'target', key: 'rust-cache']
        ]) {
          sh 'cargo test --verbose'
        }
      }
    }
  }
}
