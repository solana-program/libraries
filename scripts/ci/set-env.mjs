#!/usr/bin/env zx
import { getSolanaVersion, getToolchain } from '../utils.mjs';

await $`echo "SOLANA_VERSION=${getSolanaVersion()}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_NIGHTLY=${getToolchain('nightly')}" >> $GITHUB_ENV`;
