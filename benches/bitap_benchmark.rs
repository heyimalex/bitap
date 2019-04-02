#[macro_use]
extern crate criterion;

extern crate bitap;
extern crate bitap_reference as bref;

use criterion::black_box;
use criterion::Criterion;

static PATTERN: &'static str = "him";
static TEXT: &'static str = r#"
"Then be so kind," urged Miss Manette, "as to leave us here. You
see how composed he has become, and you cannot be afraid to leave
him with me now. Why should you be? If you will lock the door to
secure us from interruption, I do not doubt that you will find him, when
you come back, as quiet as you leave him. In any case, I will take care
of him until you return, and then we will remove him straight."

Both Mr. Lorry and Defarge were rather disinclined to this course,
and in favour of one of them remaining. But, as there were not only
carriage and horses to be seen to, but travelling papers; and as time
pressed, for the day was drawing to an end, it came at last to their hastily
dividing the business that was necessary to be done, and hurrying away
to do it.

Then, as the darkness closed in, the daughter laid her head down
on the hard ground close at the father's side, and watched him. The
darkness deepened and deepened, and they both lay quiet, until a light
gleamed through the chinks in the wall.

Mr. Lorry and Monsieur Defarge had made all ready for the jour-
ney, and had brought with them, besides travelling cloaks and wrap-
pers, bread and meat, wine, and hot coffee. Monsieur Defarge put this
provender, and the lamp he carried, on the shoemaker's bench (there
was nothing else in the garret but a pallet bed), and he and Mr. Lorry
roused the captive, and assisted him to his feet.

No human intelligence could have read the mysteries of his mind,
in the scared blank wonder of his face. Whether he knew what had
happened, whether he recollected what they had said to him, whether
he knew that he was free, were questions which no sagacity could have
solved. They tried speaking to him; but, he was so confused, and so very
slow to answer, that they took fright at his bewilderment, and agreed
for the time to tamper with him no more. He had a wild, lost man-
ner of occasionally clasping his head in his hands, that had not been
seen in him before; yet, he had some pleasure in the mere sound of his
daughter's voice, and invariably turned to it when she spoke.

In the submissive way of one long accustomed to obey under coer-
cion, he ate and drank what they gave him to eat and drink, and put on
the cloak and other wrappings, that they gave him to wear. He readily
responded to his daughter's drawing her arm through his, and took —
and kept — her hand in both his own.
"#;

fn bench_find(c: &mut Criterion) {
    c.bench_function("ref::find", move |b| {
        b.iter(|| bref::find(black_box(PATTERN), black_box(&TEXT)).unwrap())
    });
    let pattern = bitap::Pattern::new(PATTERN).unwrap();
    c.bench_function("bitap::find", move |b| {
        b.iter(|| pattern.find(black_box(&TEXT)).collect::<Vec<_>>())
    });
}

fn bench_lev(c: &mut Criterion) {
    c.bench_function("ref::lev", move |b| {
        b.iter(|| bref::lev(black_box(PATTERN), black_box(&TEXT), black_box(2)).unwrap())
    });
    let pattern = bitap::Pattern::new(PATTERN).unwrap();
    c.bench_function("bitap::lev", move |b| {
        b.iter(|| {
            pattern
                .lev(black_box(&TEXT), black_box(2))
                .collect::<Vec<_>>()
        })
    });
    let pattern = bitap::Pattern::new(PATTERN).unwrap();
    c.bench_function("bitap::lev_static", move |b| {
        b.iter(|| {
            pattern
                .lev_static(black_box(&TEXT), bitap::StaticMaxDistance::Two)
                .collect::<Vec<_>>()
        })
    });
}

fn bench_osa(c: &mut Criterion) {
    c.bench_function("ref::osa", move |b| {
        b.iter(|| bref::osa(black_box(PATTERN), black_box(&TEXT), black_box(2)).unwrap())
    });
    let pattern = bitap::Pattern::new(PATTERN).unwrap();
    c.bench_function("bitap::osa", move |b| {
        b.iter(|| {
            pattern
                .osa(black_box(&TEXT), black_box(2))
                .collect::<Vec<_>>()
        })
    });
    let pattern = bitap::Pattern::new(PATTERN).unwrap();
    c.bench_function("bitap::osa_static", move |b| {
        b.iter(|| {
            pattern
                .osa_static(black_box(&TEXT), bitap::StaticMaxDistance::Two)
                .collect::<Vec<_>>()
        })
    });
}

criterion_group!(benches, bench_find, bench_lev, bench_osa);
criterion_main!(benches);
