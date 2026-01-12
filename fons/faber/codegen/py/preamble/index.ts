/**
 * Python Preamble Generator
 *
 * Assembles preamble snippets based on features used.
 */

import type { RequiredFeatures } from '../../types';

const PRAEFIXUM = `def __praefixum__(code):
    __globals__ = {"range": range, "len": len, "list": list, "dict": dict, "int": int, "float": float, "str": str, "bool": bool, "abs": abs, "min": min, "max": max, "sum": sum}
    __locals__ = {}
    exec(code, __globals__, __locals__)
    return __locals__.get('__result__')`;

/**
 * Generate preamble based on features used.
 *
 * @param features - Feature flags set during codegen traversal
 * @returns Preamble string (empty if no features need setup)
 */
export function genPreamble(features: RequiredFeatures): string {
    const imports: string[] = [];
    const helpers: string[] = [];

    if (features.enum) {
        imports.push('from enum import Enum, auto');
    }

    if (features.decimal) {
        imports.push('from decimal import Decimal');
    }

    if (features.callable) {
        imports.push('from typing import Callable');
    }

    if (features.usesRegex) {
        imports.push('import re');
    }

    if (features.math) {
        imports.push('import math');
    }

    if (features.random) {
        imports.push('import random');
    }

    if (features.uuid) {
        imports.push('import uuid');
    }

    if (features.secrets) {
        imports.push('import secrets');
    }

    if (features.sys) {
        imports.push('import sys');
    }

    if (features.warnings) {
        imports.push('import warnings');
    }

    if (features.time) {
        imports.push('import time');
    }

    if (features.json) {
        imports.push('import json');
    }

    if (features.praefixum) {
        helpers.push(PRAEFIXUM);
    }

    const parts: string[] = [];
    if (imports.length > 0) {
        parts.push(imports.join('\n'));
    }
    if (helpers.length > 0) {
        parts.push(helpers.join('\n\n'));
    }

    return parts.length > 0 ? parts.join('\n\n') + '\n\n' : '';
}
