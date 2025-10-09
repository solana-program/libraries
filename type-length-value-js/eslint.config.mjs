// @ts-check

import eslint from '@eslint/js';
import importExtensions from 'eslint-plugin-import-extensions';
import { globalIgnores } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default tseslint.config(
    globalIgnores([
        '**/docs',
        '**/lib',
        '**/test-ledger',
        '**/package-lock.json',
    ]),
    eslint.configs.recommended,
    tseslint.configs.recommended,
    {
        plugins: {
            'import-extensions': importExtensions,
        },
        rules: {
            'import-extensions/require-extensions': 'error',
        },
    },
    {
        rules: {
            '@typescript-eslint/ban-ts-comment': 'off',
            '@typescript-eslint/no-explicit-any': 'off',
            '@typescript-eslint/no-unused-vars': 'off',
            '@typescript-eslint/no-empty-interface': 'off',
            '@typescript-eslint/consistent-type-imports': 'error',
        },
    },
    {
        files: ['examples/**/*', 'test/**/*'],
        rules: {
            'import-extensions/require-extensions': 'off',
        },
    }
);
