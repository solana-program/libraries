#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, getToolchainArgument } from '../utils.mjs';

const toolchain = getToolchainArgument('nightly');
await $`cargo ${toolchain} hack check --all-targets --feature-powerset ${cliArguments()}`;
