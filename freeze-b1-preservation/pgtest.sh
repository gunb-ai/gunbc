#!/bin/bash
PG_V='(R|A|F|X\{r=(0|[1-9]|1[0-9]|2[0-3]),a=(U|\?|-),s=(T|F),al=[0-9]{1,19}\})'
PG_OK="^WCVERIFY ctx=OK arm=${PG_V} field=${PG_V} binder=${PG_V} body=${PG_V}$"
PG_RF='^WCVERIFY ctx=RF arm=- field=- binder=- body=-$'
t(){ if [[ "$1" =~ $PG_RF ]] || [[ "$1" =~ $PG_OK ]]; then echo ACC; else echo REJ; fi; }
echo "empty-pattern behavior:"; r=x; unset PG_RF; [[ "$r" =~ $PG_RF ]] && echo "empty matches" || echo "empty no-match"
PG_RF='^WCVERIFY ctx=RF arm=- field=- binder=- body=-$'
t 'WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}'
t 'hello world'
t 'WCVERIFY ctx=RF arm=- field=- binder=- body=-'
