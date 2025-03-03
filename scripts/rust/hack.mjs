#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  popArgument,
  workingDirectory,
} from '../utils.mjs';

const toolchain = getToolchainArgument('lint');
await $`cargo ${toolchain} hack check --all-targets ${cliArguments()}`;
