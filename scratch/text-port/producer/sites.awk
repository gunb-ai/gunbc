FNR==1{imp=""; sym="-"}
/^import /{imp=imp" "$0}
/^(pub )?(fn|data|type) /{s=$0; sub(/^(pub )?(fn|data|type) /,"",s); sub(/[^A-Za-z0-9_].*/,"",s); sym=s}
{
 line=$0; gsub(/"([^"\\]|\\.)*"/,"\"\"",line); sub(/\/\/.*/,"",line)
 n=split("split replace starts_with ends_with",names," ")
 for(i=1;i<=n;i++){ nm=names[i]
   t=line; c=0
   while(match(t,"(^|[^A-Za-z0-9_.])" nm "\\(")){c++; t=substr(t,RSTART+RLENGTH)}
   for(k=0;k<c;k++){ src=(imp ~ ("[{ ,]" nm "[ ,}]"))?"imported":"ambient"; print FILENAME"\t"sym"\t"FNR"\t"nm"\t"src }
 }
}
