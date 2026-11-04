"""Bind modpack-candidate claims to exact packages and persisted runtime evidence."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path

from package_candidates import snapshot
from verify_candidate_evidence import ROOT, digest, load, require


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--scope', choices=['functional','complete'], default='complete')
    args = parser.parse_args()
    require(not args.report.exists(), 'Refusing to replace acceptance evidence')
    manifest = json.loads(args.manifest.read_text())
    build = load(manifest['buildReport'])
    require(build['status']=='passed', 'Coherent build failed')
    artifacts = {row['loader']:row['artifact'] for row in build['packages']}
    require(set(artifacts)=={'fabric','neoforge'}, 'Both loaders required')
    evidence = {}

    def record(path):
        data = load(path)
        evidence[path] = digest(ROOT/path)
        return data

    def server(data, loader):
        require(data.get('result',data.get('status'))=='passed', 'Server runtime failed')
        require(data.get('processExitCode',data.get('processExit'))==0 and not data['forcedStop'], 'Server did not stop cleanly')
        require(data['artifactUnchanged'] and data['developmentMode'] is False, 'Server artifact/development gate failed')
        # Export runs inspect registry data in a disposable vanilla world. Actual
        # generated-world save evidence is required for generation/reload trials.
        if data['qaMode']!='export':
            require(data['saveObserved'] or data.get('companionSaveCompleted'), 'Generated world save not observed: '+data['instance'])
        require(data.get('packagedArtifactOriginObserved') or data.get('artifactOriginObserved'), 'Production server origin missing')
        require(data['serverProperties']['max-tick-time']==60000, 'Normal watchdog required')
        qa = data.get('qaResult',data.get('runtimeQa'))
        require(qa['status']=='passed' and qa['runtime']['modSha256']==artifacts[loader]['sha256'], 'Wrong server artifact')
        require(qa['runtime']['modVersion']==build['version'], 'Wrong runtime version')
        return qa

    for row in build['packages']:
        require(row['sourceBefore']==row['sourceAfter']==snapshot(row['loader']), 'Package sources changed')
        require(digest(ROOT/row['artifact']['path'])==row['artifact']['sha256'], 'JAR bytes changed')
        require(not row['artifact']['qaClassesPackaged'] and row['artifact']['licenseNoticesPresent'], 'Invalid distribution contents')
    record(manifest['buildReport'])

    for loader, path in manifest['corpora'].items():
        corpus = record(path)
        require(corpus['status']=='passed' and corpus['variantCount']==17 and len(set(corpus['actualStoredAndReloadedVariants']))==17, 'Incomplete classic corpus')
        require(corpus['productionJarSha256']==artifacts[loader]['sha256'], 'Corpus package mismatch')
        for case in corpus['cases']:
            require({r['mode'] for r in case['runs']}=={'generate','reload'}, 'Missing cold reload')
            for run in case['runs']:
                server(record(run['report']),loader)
                if run['mode']=='reload':
                    require(run['reloadDigestsMatched'], 'Classic saved columns changed')
            for pillar in case.get('pillarSurfaceEvidence',[]):
                require(pillar['positiveAddedPillarColumns']>0 and pillar['maximumAddedHeight']<=pillar['heightCap'], 'Positive bounded pillars absent')

    cases = set()
    for path in manifest['compatibility']:
        summary = record(path)
        require(summary['status']=='passed' and not summary['failures'], 'Compatibility case failed')
        for row in summary['cases']:
            loader = row['loader']
            cases.add((loader,row['case']))
            require(row['status']=='passed' and row['artifactSha256']==artifacts[loader]['sha256'], 'Wrong compatibility artifact')
            require(row['consumedPackMatchesEveryExportedFile'] and row['generationDigest']==row['reloadDigest'], 'Pack/save mismatch')
            require(row['registeredClassicVariants']==17, 'Classic registry entries missing')
            for stage in ('export','generation','reload'):
                path = row[stage+'Report']
                require(digest(ROOT/path)==row[stage+'ReportSha256'], 'Compatibility raw report changed')
                server(record(path),loader)
    require(cases=={(l,c) for l in artifacts for c in ('optimized','modernfix-community','content-structures')}, 'Incomplete compatibility matrix')

    clients = set()
    for row in manifest['clients']:
        data = record(row['report'])
        client = data.get('client',data)
        qa = client['clientQa']
        runtime = qa['runtime']
        require(data.get('result',data.get('status')) in {'passed','passed-client-qa-awaiting-visual-review'}, 'Client failed')
        require(qa['status']=='passed' and client['processExitCode']==0 and client['artifactUnchanged'], 'Client shutdown/artifact failed')
        require(runtime['modSha256']==artifacts[row['loader']]['sha256'] and runtime['modVersion']==build['version'], 'Client package mismatch')
        require(runtime['developmentEnvironment'] is False and client['customTerrainValidated'] and not client['criticalRuntimeErrors'], 'Client production/terrain gate failed')
        require(client.get('packagedArtifactOriginObserved') or client.get('packagedOriginObserved'), 'Client package origin absent')
        require(qa['immediatePostDone']['safeScreenOwnership'] and not qa['immediatePostDone']['parentVisible'], 'Done race returned')
        require(qa['presetApplication']['actualDoneButton'] and qa['terrainRendering']['createdThroughActualUi'], 'Actual world creation absent')
        require(qa['terrainRendering']['customGeneratorActive'] and not qa['vanillaTerrainRendering']['customGeneratorActive'], 'World context isolation failed')
        require(qa['cacheCountInVanillaAfterCustom']==0, 'Custom caches retained in vanilla world')
        for name in ('customRenderReadiness','vanillaRenderReadiness'):
            ready = qa[name]
            require(ready['rendererComplete'] and ready['loadedInnerChunks']==ready['expectedInnerChunks'], 'Inner terrain was not rendered')
        clients.add((row['loader'],row['stack']))
    require(clients=={(l,s) for l in artifacts for s in ('base','shader-content')}, 'Incomplete client matrix')

    images = 0
    for path in manifest['imageArchives']:
        archive = record(path)
        for row in archive['reports']:
            require(digest(ROOT/row['report'])==row['reportSha256'], 'Archived report changed')
            for shot in row['images']:
                require(digest(ROOT/shot['path'])==shot['sha256'], 'Archived image changed')
                images += 1
    require(images==30, 'Expected 28 full-flow and two pillar captures')

    for loader,path in manifest['loopbacks'].items():
        data = record(path)
        require(data['status']=='passed' and data['clientJoinPassed'] and data['serverHarnessExit']==0, 'Loopback join failed')
        server(data['server'],loader)
        qa = data['client']['clientQa']
        require(qa['runtime']['modSha256']==artifacts[loader]['sha256'] and qa['networkTerrainRendering']['connectionOpen'], 'Wrong network client')
        require(len(set(qa['networkTerrainRendering']['synchronizedVariants']))==17, 'Registry sync incomplete')
    for loader, paths in manifest['linux'].items():
        for path in paths:
            server(record(path),loader)
    for loader,path in manifest['cacheLifecycle'].items():
        data = record(path)
        require(data['status']=='passed' and data['candidateChecksPassed'], 'Cache lifecycle failed')
        require(data['results']['candidate']['productionJarSha256']==artifacts[loader]['sha256'], 'Cache package mismatch')
    for loader,path in manifest['cacheScopes'].items():
        qa = server(record(path),loader)
        checks = qa['cellSamplerCacheChecks']
        require(checks['status']=='passed' and checks['rawFieldParity'] and checks['newActualHeightmapCalls']==4 and checks['oldActualHeightmapCalls']==384, 'Cell cache fidelity/call count failed')
    for loader,path in manifest['cacheReentry'].items():
        data = record(path)
        qa = server(data,loader)
        require('-Dterraforged.qa.probeCacheReentry=true' in data['jvmCommand'], 'Nested cache probe was not enabled')
        require(qa['cellSamplerCacheChecks']['status']=='passed' and qa['cellSamplerCacheChecks']['assertions']>=801, 'Nested cell cache checks failed')
    require(record(manifest['nativeColumns'])['status']=='passed', 'Native column replay failed')
    ci = record(manifest['ci'])
    require(len(ci['runs'])==2 and all(r['status']=='completed' and r['conclusion']=='success' for r in ci['runs']), 'Production source CI incomplete')

    if args.scope=='complete':
        host = record(manifest['benchmarkSession'])
        require(host['status']=='passed' and not host['compilerInterference'], 'Benchmark host interference or incomplete session')
        for path in manifest['benchmarks']:
            matrix = record(path)
            require(matrix['status']=='passed' and not matrix['budgetViolations'] and len(matrix['runs'])==12, 'Performance matrix incomplete or over budget')
            require(all(r['trials']==3 for r in matrix['summary']) and len(matrix['summary'])==4, 'Matched repeated trials absent')
            for run in matrix['runs']:
                require(run['artifactSha256']==artifacts[run['loader']]['sha256'], 'Benchmark artifact mismatch')
                require(digest(ROOT/run['report'])==run['reportSha256'], 'Benchmark raw report changed')
                server(record(run['report']),run['loader'])
                if run['trial']==1:
                    require(run['reloadColumnsVerified'], 'Benchmark disk reload missing')
                    require(digest(ROOT/run['reloadReport'])==run['reloadReportSha256'], 'Benchmark reload report changed')
                    server(record(run['reloadReport']),run['loader'])

    report = {'schemaVersion':2,'checkedAtUtc':datetime.now(timezone.utc).isoformat(),
        'status':'passed-scoped-modpack-candidate' if args.scope=='complete' else 'passed-functional-candidate-performance-pending',
        'version':build['version'],'minecraft':'26.2','artifacts':artifacts,'verifiedImages':images,
        'manifestSha256':digest(args.manifest),'evidenceSha256':evidence,'checkerSha256':digest(Path(__file__)),
        'stableRelease':False,'userVisualAcceptance':False,'horizonInfiniteGenerationReady':False,
        'limits':['New worlds and exact pinned combinations','No original Engine/classic visual parity',
            'No arbitrary packs or migration','No authenticated multiplayer/load test',
            'Far LOD rendering and FPS unaccepted','Finite local performance corpus only']}
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':report['status'],'evidenceFiles':len(evidence),'verifiedImages':images}))


if __name__=='__main__':
    main()
