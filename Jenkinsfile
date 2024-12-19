pipeline {
  agent any
  stages {
    stage('Install Rust') {
      steps {
        sh '''
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
        '''
      }
    }
    stage('Verify cargo installation') {
      steps {
        sh '''
          source $HOME/.cargo/env
          cargo --version
        '''
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
