// Curated, FROZEN name pools + decode LUTs (see design/names.md for how they
// were generated and selected). There is no generator — changing these is a
// breaking change. A syntax-constant file; keep the surrounding structure.
//
// 3-letter CVC pieces with a positional alphabet vetted by blind judging
// (no c/q; x and y are excluded from the pieces as they read as typos):
//   start consonant: b d f g h j k l m n p r s t v w z
//   end consonant:   b d f g h j k l m n p r s t v w z
//   middle vowel:    a e i o u
// A name word is <prefix><mid-vowel><suffix><end-vowel> with mid-vowel from
// PREFIX_EXTRA_VOWEL and end-vowel from SUFFIX_EXTRA_VOWEL. The Lut tables at
// the bottom map a 3-char piece back to its list index for decoding. See
// design/names.md.

pub const PREFIXES: [&str; 1024] = [
    "bab", "bad", "baf", "bag", "bah", "baj", "bak", "bal", "bam", "ban", "bap", "bar",
    "bas", "bat", "bav", "baw", "baz", "beb", "bed", "bef", "beg", "beh", "bem", "bez",
    "bib", "bid", "bif", "big", "bik", "bil", "bim", "bin", "bip", "bir", "bis", "bit",
    "biz", "bob", "bod", "bof", "bog", "boh", "bok", "bol", "bom", "bon", "bop", "bor",
    "bos", "bot", "bow", "boz", "bub", "buf", "bug", "buh", "buk", "bul", "bun", "bup",
    "buz", "dab", "dad", "daf", "dag", "dah", "dak", "dal", "dam", "dan", "dap", "dar",
    "das", "dat", "dav", "daw", "daz", "ded", "deh", "dej", "dew", "dez", "dib", "did",
    "dif", "dig", "dij", "dil", "dim", "din", "dip", "dir", "dis", "dit", "div", "diz",
    "dob", "dod", "dof", "dog", "doh", "dok", "dol", "dom", "don", "dop", "dor", "dos",
    "dot", "dov", "dow", "doz", "dub", "dud", "duf", "dug", "duh", "dum", "dup", "duz",
    "fab", "fad", "faf", "fah", "fal", "fam", "fan", "fap", "far", "fas", "fat", "fav",
    "faw", "faz", "feg", "feh", "fek", "fem", "fep", "fev", "few", "fez", "fib", "fid",
    "fif", "fig", "fij", "fil", "fim", "fin", "fip", "fir", "fis", "fit", "fiz", "fob",
    "fod", "fof", "fog", "fok", "fol", "fom", "fon", "fop", "for", "fos", "fot", "fow",
    "foz", "fub", "fud", "fuf", "fug", "fuh", "fum", "fup", "fut", "fuz", "gab", "gad",
    "gaf", "gag", "gah", "gak", "gal", "gam", "gan", "gap", "gar", "gas", "gat", "gav",
    "gaw", "gaz", "geb", "ged", "geg", "geh", "gek", "gel", "gem", "gen", "gep", "ger",
    "ges", "get", "gew", "gez", "gib", "gid", "gif", "gig", "gih", "gik", "gil", "gim",
    "gin", "gip", "gir", "gis", "git", "giz", "gob", "god", "gog", "goh", "gok", "gol",
    "gom", "gon", "gop", "gor", "gos", "got", "gov", "gow", "goz", "gub", "gud", "guf",
    "gug", "gul", "gum", "gun", "gup", "gur", "gus", "gut", "guz", "hab", "had", "haf",
    "hag", "hah", "haj", "hak", "hal", "ham", "han", "hap", "har", "has", "hat", "hav",
    "haw", "haz", "hed", "hef", "heg", "heh", "hel", "hem", "hen", "her", "hew", "hez",
    "hib", "hid", "hif", "hig", "hih", "hij", "hik", "hil", "him", "hin", "hip", "hir",
    "his", "hit", "hiz", "hob", "hod", "hof", "hog", "hoh", "hoj", "hok", "hol", "hom",
    "hon", "hop", "hor", "hos", "hot", "how", "hub", "hud", "huf", "hug", "huh", "huk",
    "hum", "hun", "hup", "hur", "huv", "jab", "jad", "jaf", "jag", "jah", "jaj", "jak",
    "jal", "jam", "jan", "jap", "jar", "jas", "jat", "jav", "jaw", "jaz", "jeb", "jed",
    "jef", "jeh", "jek", "jel", "jem", "jen", "jep", "jer", "jes", "jet", "jew", "jib",
    "jid", "jig", "jih", "jik", "jil", "jim", "jin", "jip", "jir", "jis", "jit", "jiz",
    "job", "jog", "joh", "jok", "jol", "jom", "jon", "jop", "jor", "jos", "jot", "jov",
    "jow", "jub", "jud", "jug", "juh", "juk", "jul", "jum", "jun", "jup", "jur", "jus",
    "jut", "juv", "juz", "kab", "kad", "kaf", "kag", "kah", "kak", "kal", "kam", "kan",
    "kap", "kar", "kas", "kat", "kaw", "kaz", "keb", "keh", "kek", "kel", "kem", "ken",
    "kep", "ker", "kes", "ket", "kew", "kib", "kid", "kig", "kil", "kim", "kin", "kip",
    "kir", "kis", "kit", "kiw", "kob", "kod", "kog", "koh", "kok", "kol", "kom", "kon",
    "kop", "kor", "kos", "kot", "kow", "kub", "kud", "kug", "kuh", "kul", "kun", "kup",
    "kur", "kus", "kuw", "kuz", "lab", "lad", "laf", "lag", "lah", "laj", "lak", "lal",
    "lam", "lan", "lap", "lar", "las", "lat", "lav", "law", "laz", "lef", "leh", "lel",
    "lem", "les", "lew", "lez", "lib", "lid", "lif", "lig", "lik", "lil", "lim", "lin",
    "lip", "lir", "lis", "lit", "liv", "liz", "lob", "lod", "lof", "log", "loh", "lok",
    "lol", "lom", "lon", "lop", "lor", "los", "lot", "lov", "low", "loz", "lub", "luf",
    "luh", "lum", "luv", "luz", "mab", "mad", "maf", "mag", "mah", "maj", "mak", "mal",
    "mam", "man", "map", "mar", "mas", "mat", "mav", "maw", "maz", "meh", "mej", "mem",
    "men", "mew", "mez", "mib", "mid", "mif", "mig", "mih", "mik", "mil", "mim", "min",
    "mip", "mir", "mis", "mit", "miz", "mob", "mod", "mof", "mog", "moh", "mok", "mol",
    "mom", "mon", "mop", "mor", "mos", "mot", "mov", "mow", "moz", "mub", "muf", "muh",
    "muj", "muk", "mum", "mup", "muz", "nab", "nad", "naf", "nag", "nah", "nak", "nal",
    "nam", "nan", "nap", "nar", "nas", "nat", "nav", "naw", "naz", "nef", "neg", "neh",
    "nen", "new", "nez", "nib", "nid", "nif", "nig", "nik", "nil", "nim", "nin", "nip",
    "nir", "nis", "nit", "niv", "niz", "nob", "nod", "nof", "nog", "noh", "nok", "nol",
    "nom", "non", "nop", "nor", "nos", "not", "nov", "now", "noz", "nud", "nuf", "nug",
    "nuh", "nuk", "nun", "nur", "nuz", "pab", "pad", "paf", "pag", "pah", "paj", "pak",
    "pal", "pam", "pan", "pap", "par", "pas", "pat", "pav", "paw", "paz", "peb", "ped",
    "pef", "peg", "peh", "pep", "pev", "pew", "pez", "pib", "pid", "pif", "pig", "pik",
    "pil", "pim", "pin", "pip", "pir", "pit", "piz", "pob", "pod", "poh", "pok", "pol",
    "pom", "pon", "pop", "por", "pos", "pot", "pov", "pow", "poz", "pub", "puf", "pug",
    "puh", "puk", "pul", "pum", "pup", "pus", "puz", "rab", "rad", "raf", "rag", "rah",
    "rak", "ral", "ram", "ran", "rap", "rar", "ras", "rat", "rav", "raw", "raz", "reh",
    "rer", "rew", "rez", "rib", "rid", "rif", "rig", "rih", "rik", "ril", "rim", "rin",
    "rip", "rir", "ris", "rit", "riv", "riz", "rob", "rod", "rof", "rog", "roh", "rok",
    "rol", "rom", "ron", "rop", "ror", "ros", "rot", "rov", "row", "roz", "rub", "ruf",
    "rug", "ruh", "rur", "ruz", "sab", "sad", "saf", "sag", "sah", "saj", "sak", "sal",
    "sam", "san", "sap", "sar", "sas", "sat", "sav", "saw", "saz", "seh", "sej", "ses",
    "sew", "sez", "sib", "sid", "sif", "sig", "sil", "sim", "sin", "sip", "sir", "sis",
    "sit", "siv", "siz", "sob", "sod", "sof", "sog", "soh", "sok", "sol", "som", "son",
    "sop", "sor", "sos", "sot", "sov", "sow", "soz", "suf", "suh", "sus", "suz", "tab",
    "tad", "taf", "tag", "tah", "tak", "tal", "tam", "tan", "tap", "tar", "tas", "tat",
    "tav", "taw", "taz", "tef", "teh", "tet", "tew", "tez", "tib", "tid", "tif", "tig",
    "tik", "til", "tim", "tin", "tip", "tir", "tis", "tiv", "tiz", "tob", "tod", "tof",
    "tog", "toh", "tok", "tol", "tom", "ton", "top", "tor", "tos", "tot", "tow", "tub",
    "tuf", "tum", "tup", "tur", "tut", "tuv", "tuz", "vab", "vad", "vaf", "vag", "vah",
    "vak", "val", "vam", "van", "vap", "var", "vas", "vat", "vav", "vaz", "veb", "ved",
    "veg", "veh", "vej", "vek", "vel", "vem", "ven", "vep", "ver", "ves", "vet", "vev",
    "vez", "vib", "vid", "vig", "vih", "vik", "vil", "vim", "vin", "vip", "vir", "vis",
    "vit", "viv", "viz", "vob", "vod", "vog", "vok", "vol", "vom", "von", "vop", "vor",
    "vos", "vot", "vow", "voz", "vud", "vuh", "vuk", "vul", "vun", "vur", "vus", "vut",
    "wab", "wad", "waf", "wag", "wah", "wak", "wal", "wam", "wan", "wap", "war", "was",
    "wat", "waw", "waz", "weh", "wek", "wem", "wid", "wif", "wig", "wik", "wil", "wim",
    "win", "wip", "wir", "wis", "wit", "wiz", "wob", "wok", "wol", "wom", "won", "wor",
    "wos", "wot", "wow", "wub", "wuh", "wuk", "wul", "wun", "wup", "wur", "wus", "wut",
    "zab", "zad", "zaf", "zag", "zak", "zal", "zam", "zan", "zap", "zar", "zas", "zat",
    "zaw", "zaz", "zeb", "zed", "zek", "zel", "zem", "zen", "zep", "zer", "zes", "zet",
    "zid", "zig", "zil", "zin", "zir", "zis", "zit", "zob", "zog", "zol", "zom", "zon",
    "zop", "zor", "zos", "zot", "zow", "zub", "zuf", "zuh", "zuk", "zul", "zum", "zun",
    "zup", "zur", "zus", "zut",
];

pub const SUFFIXES: [&str; 1024] = [
    "bab", "bad", "baf", "bag", "bak", "bam", "bap", "bav", "baw", "baz", "beb", "bed",
    "bef", "beg", "bej", "bek", "bel", "ben", "bep", "ber", "bes", "bet", "bev", "bew",
    "bib", "bid", "bif", "big", "bih", "bij", "bik", "bim", "bin", "bip", "bir", "bis",
    "bit", "biv", "biw", "biz", "bob", "bod", "bof", "bog", "boj", "bol", "bop", "bos",
    "bot", "bov", "boz", "bub", "bud", "buf", "bug", "buh", "buj", "buk", "bul", "bun",
    "bup", "bur", "bus", "but", "buv", "buw", "buz", "dab", "daf", "dag", "dah", "daj",
    "dak", "dam", "dan", "dap", "daz", "deb", "ded", "def", "deg", "deh", "dej", "dek",
    "del", "dem", "den", "dep", "der", "des", "det", "dev", "dew", "dib", "did", "dih",
    "dil", "dim", "din", "dis", "dit", "diw", "dob", "dod", "dof", "dog", "doh", "doj",
    "dok", "dol", "dom", "don", "dot", "dov", "doz", "dub", "dud", "duf", "dug", "duh",
    "duj", "duk", "dul", "dum", "dun", "dup", "dur", "dus", "dut", "duv", "duw", "duz",
    "fab", "fad", "faf", "fah", "faj", "fal", "fam", "fan", "fap", "fas", "fav", "faw",
    "feb", "fed", "fef", "feg", "feh", "fej", "fek", "fel", "fem", "fen", "fep", "fer",
    "fes", "fet", "fev", "few", "fib", "fid", "fif", "fih", "fij", "fim", "fip", "fir",
    "fis", "fit", "fiv", "fiw", "fob", "fod", "fof", "fog", "foh", "foj", "fok", "fol",
    "fom", "fon", "fop", "fos", "fov", "fow", "foz", "fub", "fud", "fuf", "fug", "fuh",
    "fuj", "ful", "fum", "fun", "fup", "fur", "fus", "fut", "fuv", "fuw", "fuz", "gab",
    "gaf", "gag", "gah", "gaj", "gak", "gam", "gap", "gav", "gaw", "geb", "ged", "gef",
    "geg", "geh", "gej", "gek", "gep", "gev", "gew", "gez", "gib", "gid", "gif", "gig",
    "gih", "gij", "gik", "gil", "gim", "gin", "gip", "gir", "gis", "git", "giv", "giw",
    "giz", "gob", "god", "gof", "gog", "goh", "goj", "gok", "gol", "gom", "gop", "got",
    "gub", "gud", "guf", "gug", "guh", "guj", "guk", "gul", "gum", "gun", "gup", "gur",
    "gus", "gut", "guv", "guw", "guz", "haf", "hal", "heb", "hed", "hej", "hek", "hen",
    "hep", "hes", "het", "hev", "hib", "hid", "hif", "hih", "hik", "him", "hin", "hir",
    "hit", "hiv", "hiw", "hob", "hod", "hog", "hoh", "hol", "hov", "hoz", "hud", "huf",
    "hug", "huh", "huj", "huk", "hul", "hun", "hup", "hur", "hus", "hut", "huv", "huw",
    "huz", "jab", "jad", "jaf", "jag", "jah", "jaj", "jak", "jal", "jam", "jan", "jap",
    "jar", "jas", "jat", "jav", "jaw", "jaz", "jeb", "jed", "jef", "jeg", "jeh", "jej",
    "jek", "jel", "jem", "jen", "jep", "jer", "jes", "jet", "jev", "jew", "jez", "jib",
    "jid", "jif", "jig", "jih", "jij", "jik", "jil", "jim", "jin", "jip", "jir", "jis",
    "jit", "jiv", "jiw", "jiz", "job", "jof", "jog", "joj", "jok", "jol", "jom", "jon",
    "jop", "jor", "jos", "jot", "jov", "jow", "joz", "jub", "jud", "juf", "jug", "juh",
    "juj", "juk", "jul", "jum", "jun", "jup", "jur", "jus", "jut", "juv", "juw", "juz",
    "kab", "kad", "kaf", "kag", "kaj", "kal", "kam", "kan", "kap", "kas", "kav", "keb",
    "ked", "kef", "keg", "keh", "kej", "kek", "kel", "kem", "ken", "kep", "ker", "kes",
    "ket", "kev", "kew", "kez", "kib", "kid", "kif", "kig", "kih", "kij", "kim", "kir",
    "kis", "kit", "kiv", "kiw", "kiz", "kob", "kod", "kof", "kog", "koj", "kol", "kom",
    "kon", "kop", "kor", "kos", "kot", "kov", "koz", "kub", "kud", "kuf", "kug", "kuh",
    "kuj", "kul", "kun", "kup", "kur", "kus", "kuv", "kuw", "kuz", "laf", "lal", "leb",
    "led", "leg", "lej", "lek", "len", "lep", "ler", "let", "lev", "lih", "lij", "liw",
    "lod", "loh", "loj", "lol", "lud", "luf", "lug", "luh", "luj", "luk", "lun", "lup",
    "lur", "lus", "lut", "luv", "luw", "mab", "maf", "mav", "maw", "meb", "med", "mef",
    "meg", "mek", "mel", "mep", "mer", "mes", "met", "mev", "mib", "mid", "mif", "mih",
    "mij", "mim", "mip", "miv", "miw", "miz", "mob", "mod", "mof", "mog", "moh", "moj",
    "mok", "mol", "mop", "mot", "mov", "mow", "mub", "mud", "muf", "mug", "muh", "muj",
    "muk", "mul", "mum", "mun", "mup", "mur", "mus", "mut", "muv", "muw", "muz", "nab",
    "naf", "nag", "naj", "nak", "neb", "ned", "nej", "nek", "nel", "nem", "nep", "ner",
    "nes", "net", "nev", "nib", "nih", "nij", "niw", "nob", "nod", "nog", "noh", "noj",
    "nol", "nop", "nub", "nud", "nuf", "nug", "nuh", "nuj", "nuk", "nul", "num", "nup",
    "nur", "nus", "nut", "nuv", "nuw", "nuz", "pab", "pad", "paf", "pag", "pah", "paj",
    "pak", "pal", "pam", "pan", "pat", "pav", "paw", "peb", "ped", "pef", "peg", "peh",
    "pej", "pek", "pel", "pem", "pen", "pep", "per", "pes", "pet", "pev", "pew", "pib",
    "pid", "pif", "pig", "pih", "pij", "pik", "pil", "pim", "pin", "pip", "pir", "pit",
    "piv", "piw", "piz", "pob", "pod", "pof", "pog", "poh", "poj", "pok", "pol", "pom",
    "pon", "pop", "pot", "pov", "poz", "pud", "puf", "pug", "puh", "puj", "puk", "pul",
    "pum", "pun", "pup", "pur", "pus", "puv", "puw", "puz", "raj", "reb", "red", "ref",
    "reg", "rej", "rek", "rel", "rem", "ren", "rep", "res", "ret", "rev", "rij", "riw",
    "roj", "rud", "ruh", "ruj", "ruk", "rul", "rum", "run", "rup", "rur", "rus", "rut",
    "ruv", "ruw", "sav", "seb", "sed", "sef", "seg", "seh", "sek", "sel", "sem", "sen",
    "sep", "ser", "set", "sev", "sib", "sif", "sih", "sij", "sip", "sir", "siw", "sod",
    "soh", "soj", "sok", "sow", "sub", "sud", "sug", "suh", "suj", "sul", "sum", "sun",
    "sup", "sur", "sus", "sut", "suv", "suw", "suz", "taj", "taw", "teb", "ted", "tef",
    "teg", "tej", "tek", "tel", "tem", "ten", "tep", "ter", "tes", "tet", "tev", "tib",
    "tih", "tij", "tik", "tis", "tiw", "tob", "toh", "toj", "tol", "tov", "toz", "tud",
    "tuf", "tug", "tuh", "tuj", "tuk", "tul", "tum", "tun", "tup", "tur", "tus", "tuw",
    "vab", "vad", "vaf", "vag", "vah", "vaj", "vak", "val", "vam", "van", "vap", "var",
    "vas", "vat", "vav", "vaw", "vaz", "veb", "ved", "vef", "veg", "veh", "vej", "vek",
    "vel", "vem", "ven", "vep", "ves", "vet", "vev", "vew", "vib", "vid", "vif", "vig",
    "vih", "vij", "vik", "vil", "vim", "vin", "vip", "vir", "vis", "vit", "viv", "viw",
    "viz", "vob", "vod", "vof", "vog", "voh", "voj", "vok", "vol", "vom", "von", "vop",
    "vor", "vos", "vot", "vov", "vow", "voz", "vub", "vud", "vuf", "vug", "vuh", "vuj",
    "vuk", "vul", "vum", "vun", "vup", "vur", "vus", "vut", "vuv", "vuw", "vuz", "wab",
    "wad", "waf", "wag", "waj", "wam", "wan", "wap", "was", "wat", "wav", "waw", "waz",
    "web", "wed", "wef", "weg", "weh", "wej", "wek", "wel", "wem", "wen", "wep", "wer",
    "wes", "wet", "wev", "wew", "wez", "wib", "wid", "wif", "wig", "wih", "wij", "wil",
    "wim", "win", "wip", "wir", "wis", "wit", "wiv", "wiw", "wiz", "wob", "wod", "wof",
    "wog", "woh", "woj", "wok", "wol", "wom", "won", "wor", "wos", "wot", "wov", "woz",
    "wub", "wud", "wuf", "wug", "wuh", "wuj", "wuk", "wul", "wum", "wun", "wup", "wur",
    "wus", "wut", "wuv", "wuw", "wuz", "zab", "zaf", "zag", "zah", "zaj", "zak", "zal",
    "zam", "zan", "zap", "zas", "zat", "zav", "zaw", "zaz", "zeb", "zed", "zef", "zeg",
    "zeh", "zej", "zek", "zel", "zem", "zen", "zep", "zer", "zes", "zet", "zev", "zew",
    "zez", "zib", "zid", "zif", "zig", "zih", "zij", "zik", "zil", "zim", "zin", "zip",
    "zir", "zis", "zit", "ziv", "ziw", "ziz", "zob", "zod", "zof", "zog", "zoh", "zoj",
    "zok", "zol", "zom", "zon", "zop", "zor", "zos", "zot", "zov", "zow", "zoz", "zub",
    "zud", "zuf", "zug", "zuh", "zuj", "zuk", "zul", "zum", "zun", "zup", "zur", "zus",
    "zut", "zuv", "zuw", "zuz",
];

/// Mid-word vowel appended after a prefix (no silent `e`).
pub const PREFIX_EXTRA_VOWEL: [u8; 4] = *b"iaou";

/// Word-end vowel; `y` preferred over silent `e`.
pub const SUFFIX_EXTRA_VOWEL: [u8; 4] = *b"yaou";


// Decode lookup tables: map a 3-char CVC piece (packed as its 3 bytes) to its
// index in PREFIXES / SUFFIXES.
pub(crate) const SLOT: usize = 1 << 10; // 1024-piece pools (10-bit index)
pub(crate) const SLOT_MASK: u64 = (SLOT as u64) - 1;

/// Slots: (packed_key, index). packed_key == u32::MAX means empty.
pub(crate) struct Lut<const N: usize> {
    slots: [(u32, u16); N],
}

impl<const N: usize> Lut<N> {
    pub(crate) const fn build(list: &[&str; SLOT]) -> Self {
        let mut slots = [(u32::MAX, 0u16); N];
        let mut i = 0usize;
        while i < list.len() {
            let b = list[i].as_bytes();
            let key = (b[0] as u32)
                | ((b[1] as u32) << 8)
                | ((b[2] as u32) << 16);
            let mut slot = ((key ^ (key >> 16)) as usize) & (N - 1);
            loop {
                if slots[slot].0 == u32::MAX {
                    slots[slot] = (key, i as u16);
                    break;
                }
                slot = (slot + 1) & (N - 1);
            }
            i += 1;
        }
        Self { slots }
    }

    pub(crate) fn lookup(&self, s: &str) -> Option<u16> {
        if s.len() != 3 {
            return None;
        }
        let b = s.as_bytes();
        let key = (b[0] as u32)
            | ((b[1] as u32) << 8)
            | ((b[2] as u32) << 16);
        let mut slot = ((key ^ (key >> 16)) as usize) & (N - 1);
        loop {
            let (k, v) = self.slots[slot];
            if k == u32::MAX {
                return None;
            }
            if k == key {
                return Some(v);
            }
            slot = (slot + 1) & (N - 1);
        }
    }
}

// N = 2048: power of two, > 1024, keeps load factor < 0.5.
pub(crate) static PREFIX_LUT: Lut<2048> = Lut::build(&PREFIXES);
pub(crate) static SUFFIX_LUT: Lut<2048> = Lut::build(&SUFFIXES);


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const BANNED: [&str; 27] = [
        "cum", "cun", "cok", "cuz", "dik", "dyk", "fag", "fak", "fik",
        "fuc", "fuk", "jod", "kik", "kuk", "kum", "kut", "lul", "pis",
        "put", "sex", "sic", "sik", "suk", "sux", "tit", "wop", "bum",
    ];

    #[test]
    fn lists_are_sized_and_unique() {
        assert_eq!(PREFIXES.len(), SUFFIXES.len());
        for list in [PREFIXES.as_slice(), SUFFIXES.as_slice()] {
            let set: HashSet<&str> = list.iter().copied().collect();
            assert_eq!(set.len(), list.len(), "duplicate piece");
        }
        assert_eq!(PREFIX_EXTRA_VOWEL, *b"iaou");
        assert_eq!(SUFFIX_EXTRA_VOWEL, *b"yaou");
    }

    #[test]
    fn pieces_obey_the_positional_alphabet() {
        let start: HashSet<char> = "bdfghjklmnprstvwz".chars().collect();
        let end: HashSet<char> = "bdfghjklmnprstvwz".chars().collect();
        let vow: HashSet<char> = "aeiou".chars().collect();
        for t in PREFIXES.iter().chain(SUFFIXES.iter()) {
            let b = t.as_bytes();
            assert_eq!(b.len(), 3, "bad length: {t}");
            assert!(start.contains(&(b[0] as char)), "bad start: {t}");
            assert!(vow.contains(&(b[1] as char)), "bad vowel: {t}");
            assert!(end.contains(&(b[2] as char)), "bad end: {t}");
        }
    }

    #[test]
    fn no_banned_pieces() {
        for t in PREFIXES.iter().chain(SUFFIXES.iter()) {
            for bad in BANNED {
                assert!(!t.contains(bad), "banned substring {bad} in {t}");
            }
        }
    }

    #[test]
    fn overlap_is_bounded() {
        let a: HashSet<&str> = PREFIXES.iter().copied().collect();
        let shared = SUFFIXES.iter().filter(|s| a.contains(**s)).count();
        assert!(
            shared <= 700,
            "overlap {shared} grew beyond the documented bound (anti-phishing cost)"
        );
    }
}
