# usage: longbench.py URL KEY MODEL OUT.jsonl MINUTES N1,N2,...
# Repeats rounds of N concurrent 512-token generations; logs aggregate tok/s per round.
import json,sys,time,threading,urllib.request
url,key,model,out,mins,Ns=sys.argv[1],sys.argv[2],sys.argv[3],sys.argv[4],float(sys.argv[5]),[int(x) for x in sys.argv[6].split(',')]
def req(i,res):
    body=json.dumps({"model":model,"messages":[{"role":"user","content":f"Essay #{i} ({time.time()}): write a detailed essay on the history of computing."}],"max_tokens":512,"ignore_eos":True,"temperature":0.7}).encode()
    r=urllib.request.Request(url+"/v1/chat/completions",body,{"Content-Type":"application/json","Authorization":"Bearer "+key})
    try:
        t=time.time(); j=json.load(urllib.request.urlopen(r,timeout=600)); res[i]=(j["usage"]["completion_tokens"],time.time()-t)
    except Exception as e: res[i]=(0,0); print("err",e,flush=True)
end=time.time()+mins*60
with open(out,"a") as f:
    while time.time()<end:
        for n in Ns:
            res={}; th=[threading.Thread(target=req,args=(i,res)) for i in range(n)]
            t=time.time(); [x.start() for x in th]; [x.join() for x in th]; w=time.time()-t
            tok=sum(v[0] for v in res.values())
            f.write(json.dumps({"ts":time.time(),"n":n,"tok":tok,"wall":w,"agg":tok/w,"per":[v[0]/v[1] for v in res.values() if v[1]]})+"\n"); f.flush()
