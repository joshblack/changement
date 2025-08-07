#!/usr/bin/env node

const { changementMainNapi } = require('./index.js');

// Get command line arguments (excluding node and script name)
const args = process.argv.slice(2);

try {
  const result = changementMainNapi(args);
  console.log(result);
} catch (error) {
  console.error('Error:', error.message);
  process.exit(1);
}