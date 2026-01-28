#!/usr/bin/env bun

/**
 * Rivus Feature Usage Analyzer
 *
 * Parses the rivus codebase and counts AST node types to show
 * which language features are actually used and how often.
 */

import { $ } from 'bun';
import { Glob } from 'bun';

type Counts = Map<string, number>;

const FEATURES: Record<string, string[]> = {
    'Declarations': [
        'varia', 'fixum',
        'functio', 'functio async', 'functio*', 'functio async*', 'functio abstracta', 'structor',
        'genus', 'genus abstractum', 'genus sub', 'genus implet',
        'ordo', 'discretio', 'pactum', 'typus',
        'importa {}', 'importa *', 'ex..destructura', '[..destructura]',
    ],
    'Control Flow': [
        'si', 'si..secus', 'si..sin',
        'dum', 'fac', 'fac..dum',
        'ex..fixum', 'ex..varia', 'ex..fixum cede', 'ex..varia cede',
        'de..fixum', 'de..varia', 'de..fixum cede', 'de..varia cede',
        'elige', 'discerne', 'discerne omnia', 'custodi',
    ],
    'Actions': [
        'redde', 'redde <expr>',
        'scribe', 'vide', 'mone',
        'rumpe', 'perge', 'tacet',
        'iace', 'mori', 'tempta', 'adfirma',
        'cura', 'ad', 'incipit', 'incipiet',
    ],
    'Literals': [
        'numerus', 'fractus', 'textus', 'exemplar', 'regex',
        'verum', 'falsum', 'nihil',
        'identifier', 'ego',
    ],
    'Operators': [
        'binaria: +', 'binaria: -', 'binaria: *', 'binaria: /', 'binaria: %',
        'binaria: ==', 'binaria: !=', 'binaria: ===', 'binaria: !==',
        'binaria: <', 'binaria: >', 'binaria: <=', 'binaria: >=',
        'binaria: et', 'binaria: aut', 'binaria: vel',
        'binaria: &&', 'binaria: ||', 'binaria: ??',
        'binaria: inter', 'binaria: intra',
        'binaria: &', 'binaria: |', 'binaria: ^',
        'unaria: -', 'unaria: !', 'unaria: ~',
        'unaria: nihil', 'unaria: nonnihil', 'unaria: positivum',
        'assignatio: =', 'assignatio: +=', 'assignatio: -=', 'assignatio: *=', 'assignatio: /=', 'assignatio: %=',
        'assignatio: &&=', 'assignatio: ||=', 'assignatio: ??=',
    ],
    'Expressions': [
        'call: ()', 'call: ?.()',
        'member: .prop', 'member: ?.prop', 'member: !.prop', 'member: [expr]',
        '[..]', '{..}',
        'qua', 'innatum', 'est', 'finge',
        'cede', 'novum', 'clausura',
        'sic..secus', 'scriptum()',
        '..', 'usque',
        'dextratum', 'sinistratum',
        'numeratum', 'fractatum', 'textatum', 'bivalentum',
        'sparge', 'praefixum',
        'lege', 'lege lineam',
        'ab',
    ],
    'Tests': [
        'probandum', 'proba', 'praepara', 'postpara',
    ],
};

function inc(counts: Counts, key: string): void {
    counts.set(key, (counts.get(key) || 0) + 1);
}

function walkAndCount(node: unknown, counts: Counts): void {
    if (node === null || node === undefined) return;

    if (Array.isArray(node)) {
        for (const item of node) walkAndCount(item, counts);
        return;
    }

    if (typeof node !== 'object') return;

    const obj = node as Record<string, unknown>;
    const tag = obj.tag as string | undefined;

    if (tag) {
        switch (tag) {
            case 'VariaSententia': {
                const species = obj.species as number;
                const names = ['varia', 'fixum'];
                inc(counts, names[species] || `varia[${species}]`);
                break;
            }

            case 'IteratioSententia': {
                const species = obj.species as number;
                const mutabilis = obj.mutabilis as boolean;
                const asynca = obj.asynca as boolean;
                const prefix = species === 0 ? 'ex' : 'de';
                const binding = mutabilis ? 'varia' : 'fixum';
                const async = asynca ? ' cede' : '';
                inc(counts, `${prefix}..${binding}${async}`);
                break;
            }

            case 'ReddeSententia': {
                const hasValue = obj.valor !== null;
                inc(counts, hasValue ? 'redde <expr>' : 'redde');
                break;
            }

            case 'Littera': {
                const species = obj.species as number;
                const names = ['numerus', 'fractus', 'textus', 'exemplar', 'verum', 'falsum', 'nihil'];
                inc(counts, names[species] || `littera[${species}]`);
                break;
            }

            case 'LitteraExemplar':
                inc(counts, 'exemplar');
                break;

            case 'LitteraRegex':
                inc(counts, 'regex');
                break;

            case 'BinariaExpressia': {
                const signum = obj.signum as string;
                inc(counts, `binaria: ${signum}`);
                break;
            }

            case 'UnariaExpressia': {
                const signum = obj.signum as string;
                inc(counts, `unaria: ${signum}`);
                break;
            }

            case 'AssignatioExpressia': {
                const signum = obj.signum as string;
                inc(counts, `assignatio: ${signum}`);
                break;
            }

            case 'ConversioExpressia': {
                const signum = obj.signum as string;
                inc(counts, signum);
                break;
            }

            case 'TranslatioExpressia': {
                const directio = obj.directio as string;
                inc(counts, directio);
                break;
            }

            case 'ScribeSententia': {
                const gradus = obj.gradus as number;
                const names = ['scribe', 'vide', 'mone'];
                inc(counts, names[gradus] || `scribe[${gradus}]`);
                break;
            }

            case 'ImportaSententia': {
                const totum = obj.totum as boolean;
                inc(counts, totum ? 'importa *' : 'importa {}');
                break;
            }

            case 'FunctioDeclaratio': {
                const asynca = obj.asynca as boolean;
                const generator = obj.generator as boolean;
                const structor = obj.structor as boolean;
                const abstracta = obj.abstracta as boolean;
                if (structor) inc(counts, 'structor');
                else if (abstracta) inc(counts, 'functio abstracta');
                else if (asynca && generator) inc(counts, 'functio async*');
                else if (asynca) inc(counts, 'functio async');
                else if (generator) inc(counts, 'functio*');
                else inc(counts, 'functio');
                break;
            }

            case 'GenusDeclaratio': {
                const abstractum = obj.abstractum as boolean;
                const extendit = obj.extendit as string | null;
                const implet = obj.implet as string[] | null;
                if (abstractum) inc(counts, 'genus abstractum');
                else if (extendit) inc(counts, 'genus sub');
                else if (implet && implet.length > 0) inc(counts, 'genus implet');
                else inc(counts, 'genus');
                break;
            }

            case 'SiSententia': {
                const alternans = obj.alternans as unknown;
                const altTag = alternans && typeof alternans === 'object' ? (alternans as Record<string, unknown>).tag : null;
                if (!alternans) inc(counts, 'si');
                else if (altTag === 'SiSententia') inc(counts, 'si..sin');
                else inc(counts, 'si..secus');
                break;
            }

            case 'MembrumExpressia': {
                const computatum = obj.computatum as boolean;
                const optivum = obj.optivum as boolean;
                const nonNullum = obj.nonNullum as boolean;
                if (optivum) inc(counts, 'member: ?.prop');
                else if (nonNullum) inc(counts, 'member: !.prop');
                else if (computatum) inc(counts, 'member: [expr]');
                else inc(counts, 'member: .prop');
                break;
            }

            case 'VocatioExpressia': {
                const optivum = obj.optivum as boolean;
                inc(counts, optivum ? 'call: ?.()' : 'call: ()');
                break;
            }

            case 'DiscerneSententia': {
                const exhaustiva = obj.exhaustiva as boolean;
                inc(counts, exhaustiva ? 'discerne omnia' : 'discerne');
                break;
            }

            case 'IaceSententia': {
                const fatale = obj.fatale as boolean;
                inc(counts, fatale ? 'mori' : 'iace');
                break;
            }

            case 'FacSententia': {
                const condicio = obj.condicio as unknown;
                inc(counts, condicio ? 'fac..dum' : 'fac');
                break;
            }

            case 'AmbitusExpressia': {
                const inclusivum = obj.inclusivum as boolean;
                inc(counts, inclusivum ? 'usque' : '..');
                break;
            }

            case 'DispersioElementum':
                inc(counts, 'sparge');
                break;

            case 'LegeExpressia': {
                const modus = obj.modus as string;
                inc(counts, modus === 'line' ? 'lege lineam' : 'lege');
                break;
            }

            case 'PraeparaMassa': {
                const tempus = obj.tempus as number;
                inc(counts, tempus === 0 ? 'praepara' : 'postpara');
                break;
            }

            case 'DumSententia': inc(counts, 'dum'); break;
            case 'EligeSententia': inc(counts, 'elige'); break;
            case 'TemptaSententia': inc(counts, 'tempta'); break;
            case 'CustodiSententia': inc(counts, 'custodi'); break;
            case 'AdfirmaSententia': inc(counts, 'adfirma'); break;
            case 'RumpeSententia': inc(counts, 'rumpe'); break;
            case 'PergeSententia': inc(counts, 'perge'); break;
            case 'TacetSententia': inc(counts, 'tacet'); break;
            case 'OrdoDeclaratio': inc(counts, 'ordo'); break;
            case 'DiscretioDeclaratio': inc(counts, 'discretio'); break;
            case 'PactumDeclaratio': inc(counts, 'pactum'); break;
            case 'TypusAliasDeclaratio': inc(counts, 'typus'); break;
            case 'IncipitSententia': inc(counts, 'incipit'); break;
            case 'IncipietSententia': inc(counts, 'incipiet'); break;
            case 'CuraSententia': inc(counts, 'cura'); break;
            case 'AdSententia': inc(counts, 'ad'); break;
            case 'DestructuraSententia': inc(counts, 'ex..destructura'); break;
            case 'SeriesDestructuraSententia': inc(counts, '[..destructura]'); break;
            case 'ProbandumSententia': inc(counts, 'probandum'); break;
            case 'ProbaSententia': inc(counts, 'proba'); break;
            case 'QuaExpressia': inc(counts, 'qua'); break;
            case 'InnatumExpressia': inc(counts, 'innatum'); break;
            case 'EstExpressia': inc(counts, 'est'); break;
            case 'FingeExpressia': inc(counts, 'finge'); break;
            case 'CedeExpressia': inc(counts, 'cede'); break;
            case 'NovumExpressia': inc(counts, 'novum'); break;
            case 'ClausuraExpressia': inc(counts, 'clausura'); break;
            case 'SeriesExpressia': inc(counts, '[..]'); break;
            case 'ObiectumExpressia': inc(counts, '{..}'); break;
            case 'CondicioExpressia': inc(counts, 'sic..secus'); break;
            case 'ScriptumExpressia': inc(counts, 'scriptum()'); break;
            case 'EgoExpressia': inc(counts, 'ego'); break;
            case 'Nomen': inc(counts, 'identifier'); break;
            case 'PraefixumExpressia': inc(counts, 'praefixum'); break;
            case 'AbExpressia': inc(counts, 'ab'); break;
        }
    }

    for (const value of Object.values(obj)) {
        walkAndCount(value, counts);
    }
}

async function parseFile(path: string): Promise<unknown | null> {
    try {
        const result = await $`opus/bin/rivus parse --compact --input ${path} 2>/dev/null`.text();
        return JSON.parse(result);
    } catch {
        return null;
    }
}

function printTable(title: string, features: string[], counts: Counts, total: number, showUnused: boolean): void {
    const used = features.filter(f => (counts.get(f) || 0) > 0).map(f => [f, counts.get(f)!] as [string, number]);
    const unused = features.filter(f => !counts.has(f) || counts.get(f) === 0);

    used.sort((a, b) => b[1] - a[1]);

    console.log(`\n## ${title}\n`);

    if (used.length > 0) {
        console.log('| Feature | Count | % |');
        console.log('|---------|------:|--:|');
        for (const [key, count] of used) {
            const pct = ((count / total) * 100).toFixed(1);
            console.log(`| ${key} | ${count.toLocaleString()} | ${pct}% |`);
        }
    }

    if (showUnused && unused.length > 0) {
        console.log(`\n**Unused:** ${unused.join(', ')}`);
    }
}

async function main() {
    const showUnused = process.argv.includes('--unused') || process.argv.includes('-u');

    const counts: Counts = new Map();
    const glob = new Glob('fons/rivus/**/*.fab');
    const files: string[] = [];

    for await (const path of glob.scan('.')) {
        files.push(path);
    }

    console.error(`Parsing ${files.length} files...\n`);

    let parsed = 0;
    let failed = 0;

    for (const path of files) {
        const ast = await parseFile(path);
        if (ast) {
            walkAndCount(ast, counts);
            parsed++;
        } else {
            failed++;
            console.error(`  Failed: ${path}`);
        }
    }

    console.error(`\nParsed: ${parsed}, Failed: ${failed}\n`);

    const allFeatures = Object.values(FEATURES).flat();
    const total = allFeatures.reduce((sum, f) => sum + (counts.get(f) || 0), 0);

    console.log('# Rivus Feature Usage\n');
    console.log(`Total: ${total.toLocaleString()} feature occurrences`);

    for (const [category, features] of Object.entries(FEATURES)) {
        printTable(category, features, counts, total, showUnused);
    }

    if (showUnused) {
        const allUnused = allFeatures.filter(f => !counts.has(f) || counts.get(f) === 0);
        console.log(`\n---\n**Total unused features:** ${allUnused.length} / ${allFeatures.length}`);
    }
}

main();
