#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Get command line arguments (excluding node and script name)
const args = process.argv.slice(2);

// Find the Rust binary - this should be built and available in the package
// The binary should be included in the npm package distribution
const binaryName = process.platform === 'win32' ? 'changement.exe' : 'changement';
const binaryPath = path.join(__dirname, 'bin', binaryName);

// Execute the Rust binary with the provided arguments
const child = spawn(binaryPath, args, { 
  stdio: 'inherit' 
});

child.on('error', (error) => {
  console.error('Error executing changement:', error.message);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code || 0);
});