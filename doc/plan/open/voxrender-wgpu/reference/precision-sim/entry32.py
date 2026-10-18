import math, random, struct
from tie import F32

F=13; DB=16          # origin fraction bits, direction magnitude bits
ONE=1<<F
WINDOW=2              # cells a guess may miss by before an error term could leave i32

def wrap32(x):        # two's-complement i32 wrap, as WGSL/SPIR-V integer ops do
    x&=0xffffffff
    return x-(1<<32) if x>=1<<31 else x

def i32(x):
    assert -(1<<31)<=x<(1<<31), x
    return x

def ref_entry(O, D, B):
    # mirrors PlacementWalk::new from render_ray_walk.rs in unbounded integers:
    # (entry axis, start cell, error terms E01 E02 E12) or None on a miss
    O=[B[a]*ONE-O[a] if D[a]<0 else O[a] for a in range(3)]; D=[abs(c) for c in D]
    for a in range(3):
        if D[a]==0 and not 0<=O[a]<B[a]*ONE: return None
    e=None
    for a in range(3):
        if D[a]>0 and O[a]<0 and not (e is not None and -O[e]*D[a]>-O[a]*D[e]): e=a
    cell=[]
    for a in range(3):
        if a==e: cell.append(-1)
        elif e is not None and D[a]>0:
            s=O[a]*D[e]-O[e]*D[a]; span=D[e]*ONE; c=s//span
            cell.append(c-1 if a>e and s%span==0 else c)
        else: cell.append(O[a]>>F)
    if not all(a==e or 0<=cell[a]<B[a] for a in range(3)): return None
    nxt=lambda a: (cell[a]+1)*ONE-O[a]
    E=tuple(nxt(a)*D[b]-nxt(b)*D[a] for a,b in ((0,1),(0,2),(1,2)))
    return e, tuple(cell), tuple(i32(v) for v in E)

def nudge(x, ulps, rng):
    # moves an f32 by up to `ulps` units in the last place, for GPU ops looser than RTE
    if ulps==0 or x==0: return x
    b=struct.unpack('I', struct.pack('f', x))[0]+rng.randint(-ulps, ulps)
    return F32(struct.unpack('f', struct.pack('I', b&0xffffffff))[0])

def entry32(O, D, B, ulps=0, rng=None, offset=(0,0,0), stats=None):
    # the GPU's entry: only i32 inputs, wrapping i32 integer ops, and f32 guesses
    O=[i32(B[a]*ONE-O[a]) if D[a]<0 else O[a] for a in range(3)]; D=[abs(c) for c in D]
    mul=lambda x,y: wrap32(x*y); add=lambda x,y: wrap32(x+y); sub=lambda x,y: wrap32(x-y)
    for a in range(3):
        if D[a]==0 and not 0<=O[a]<B[a]*ONE: return None
    entering=[a for a in range(3) if D[a]>0 and O[a]<0]
    if not entering:
        cell=[O[a]>>F for a in range(3)]
        if not all(0<=cell[a]<B[a] for a in range(3)): return None
        e=None
    else:
        f=lambda x: nudge(F32(x), ulps, rng)
        t={a: f(f(-F32(O[a]))/F32(D[a])) for a in entering}
        e=max(entering, key=lambda a: t[a])
        while True:
            cell=[0]*3; cell[e]=-1; behind=None
            for a in range(3):
                if a==e: continue
                if D[a]==0: cell[a]=O[a]>>F; continue
                # f32 guess of the ray's cell on `a` where it reaches the plane of `e`
                x=f(F32(O[a])+f(f(-F32(O[e]))/F32(D[e]))*F32(D[a]))/ONE
                if x>B[a]+WINDOW: return None
                assert x>-WINDOW-1, x
                c=int(math.floor(x))+offset[a]
                # r = O_a D_e - O_e D_a - c ONE D_e is exact mod 2^32 and within i32 near the ray
                span=ONE*D[e]
                r=sub(sub(mul(O[a],D[e]),mul(O[e],D[a])),mul(c,span))
                moves=0
                while r<0 or (r==0 and a>e): c-=1; r=add(r,span); moves+=1
                while r>span or (r==span and a<e): c+=1; r=sub(r,span); moves+=1
                assert moves<=WINDOW, moves
                if stats is not None and moves: stats['corrected']+=1
                cell[a]=c
                if a in entering and c<0: behind=a
            if behind is None: break
            # the ray had not yet crossed `behind` at the guessed face: it enters later, through `behind`
            if stats is not None: stats['switched']+=1
            e=behind
        if not all(a==e or 0<=cell[a]<B[a] for a in range(3)): return None
    nxt=lambda a: sub(mul(cell[a]+1,ONE),O[a])
    E=tuple(sub(mul(nxt(a),D[b]),mul(nxt(b),D[a])) for a,b in ((0,1),(0,2),(1,2)))
    return e, tuple(cell), E

def quant_dir(d):
    m=max(abs(c) for c in d)
    return tuple(int(round(c/m*((1<<DB)-1))) for c in d)

def origin_range(B):
    # an origin and its mirror both fit i32
    return max(B)*ONE-(1<<31)+1, (1<<31)-1-max(B)*ONE

rng=random.Random(7)
def rays():
    for _ in range(40000):
        B=tuple(rng.choice((1,3,16,64,255,1024,4096)) for _ in range(3))
        lo,hi=origin_range(B)
        reach=rng.choice((4,64,1<<10,1<<14,1<<17,1<<18))*ONE
        O=tuple(max(lo,min(hi,rng.randint(-reach,reach))) for _ in range(3))
        kind=rng.random()
        if kind<0.4:   # at a random point near the grid
            T=[rng.randint(-2*ONE,(B[a]+2)*ONE) for a in range(3)]
        elif kind<0.8: # at a lattice point on the grid's faces, edges and corners: exact ties
            T=[rng.choice((0,B[a],rng.randint(0,B[a])))*ONE for a in range(3)]
        else:          # anywhere, including axis-aligned and away from the grid
            T=[O[a]+rng.choice((0,rng.randint(-reach,reach))) for a in range(3)]
        d=[T[a]-O[a] for a in range(3)]
        if d==[0,0,0]: continue
        yield O, quant_dir(d), B

n=hits=mismatch=0
stats={'corrected':0,'switched':0}
for O,D,B in rays():
    want=ref_entry(O,D,B)
    got=[entry32(O,D,B,stats=stats)]
    got+=[entry32(O,D,B,ulps=3,rng=rng) for _ in range(2)]
    # any guess within the window lands on the same start
    got+=[entry32(O,D,B,offset=tuple(rng.randint(-1,1) for _ in range(3))) for _ in range(2)]
    n+=1; hits+=want is not None; mismatch+=any(g!=want for g in got)
print(f"rays={n} hits={hits} start mismatches={mismatch} corrected={stats['corrected']} switched={stats['switched']}")
