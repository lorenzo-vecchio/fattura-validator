//! XML ingestion benchmarks.
//!
//! The 20-line document matches the shape and size class of the performance
//! budget in `PLAN §2.4`; the 5 MB case exercises the large-file budget. These
//! benchmarks measure the ingestion layer only. End-to-end budgets are owned by
//! E9.4 and deliberately live there, not here.
//!
//! Run with `cargo bench -p fattura-core --bench xml`.

// Criterion's macros expand to an undocumented `fn benches()`; benchmarks are
// not part of the public API surface.
#![allow(missing_docs, clippy::expect_used, clippy::arithmetic_side_effects)]

use std::fmt::Write as _;
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fattura_core::xml::{Event, Limits, Reader, scan};

const ROOT_OPEN: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    r#"<p:FatturaElettronica versione="FPR12" SistemaEmittente="BENCH""#,
    r#" xmlns:p="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2""#,
    r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
    r#" xsi:schemaLocation="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2 Schema_VFPR12_v1.2.3.xsd">"#,
    r#"<FatturaElettronicaHeader><DatiTrasmissione><IdTrasmittente>"#,
    r#"<IdPaese>IT</IdPaese><IdCodice>01234567890</IdCodice></IdTrasmittente>"#,
    r#"<ProgressivoInvio>00001</ProgressivoInvio><FormatoTrasmissione>FPR12</FormatoTrasmissione>"#,
    r#"<CodiceDestinatario>0000000</CodiceDestinatario></DatiTrasmissione>"#,
    r#"<CedentePrestatore><DatiAnagrafici><IdFiscaleIVA>"#,
    r#"<IdPaese>IT</IdPaese><IdCodice>01234567890</IdCodice></IdFiscaleIVA>"#,
    r#"<Anagrafica><Denominazione>Fornitore di prova</Denominazione></Anagrafica>"#,
    r#"<RegimeFiscale>RF01</RegimeFiscale></DatiAnagrafici>"#,
    r#"<Sede><Indirizzo>Via Roma 1</Indirizzo><CAP>00100</CAP><Comune>Roma</Comune>"#,
    r#"<Nazione>IT</Nazione></Sede></CedentePrestatore>"#,
    r#"<CessionarioCommittente><DatiAnagrafici><Anagrafica>"#,
    r#"<Denominazione>Cliente di prova</Denominazione></Anagrafica></DatiAnagrafici>"#,
    r#"<Sede><Indirizzo>Via Milano 2</Indirizzo><CAP>20100</CAP><Comune>Milano</Comune>"#,
    r#"<Nazione>IT</Nazione></Sede></CessionarioCommittente></FatturaElettronicaHeader>"#,
    r#"<FatturaElettronicaBody><DatiGenerali><DatiGeneraliDocumento>"#,
    r#"<TipoDocumento>TD01</TipoDocumento><Divisa>EUR</Divisa><Data>2026-11-18</Data>"#,
    r#"<Numero>1/2026</Numero></DatiGeneraliDocumento></DatiGenerali>"#,
    r#"<DatiBeniServizi>"#,
);

const BODY_CLOSE: &str = concat!(
    r#"<DatiRiepilogo><AliquotaIVA>22.00</AliquotaIVA>"#,
    r#"<ImponibileImporto>2000.00</ImponibileImporto><Imposta>440.00</Imposta>"#,
    r#"</DatiRiepilogo></DatiBeniServizi></FatturaElettronicaBody></p:FatturaElettronica>"#,
);

fn invoice_with_lines(lines: usize) -> String {
    let mut document = String::with_capacity(ROOT_OPEN.len() + lines * 180 + BODY_CLOSE.len());
    document.push_str(ROOT_OPEN);
    for index in 1..=lines {
        write!(
            document,
            "<DettaglioLinee><NumeroLinea>{index}</NumeroLinea>\
             <Descrizione>Servizio di consulenza numero {index}</Descrizione>\
             <Quantita>1.00</Quantita><PrezzoUnitario>100.00</PrezzoUnitario>\
             <PrezzoTotale>100.00</PrezzoTotale><AliquotaIVA>22.00</AliquotaIVA>\
             </DettaglioLinee>"
        )
        .expect("writing to a String cannot fail");
    }
    document.push_str(BODY_CLOSE);
    document
}

fn invoice_with_approx_bytes(bytes: usize) -> String {
    let mut document = String::with_capacity(bytes.saturating_add(BODY_CLOSE.len()));
    document.push_str(ROOT_OPEN);
    let mut index = 1usize;
    while document.len() < bytes {
        write!(
            document,
            "<DettaglioLinee><NumeroLinea>{index}</NumeroLinea>\
             <Descrizione>Servizio di consulenza numero {index}</Descrizione>\
             <Quantita>1.00</Quantita><PrezzoUnitario>100.00</PrezzoUnitario>\
             <PrezzoTotale>100.00</PrezzoTotale><AliquotaIVA>22.00</AliquotaIVA>\
             </DettaglioLinee>"
        )
        .expect("writing to a String cannot fail");
        index = index.saturating_add(1);
    }
    document.push_str(BODY_CLOSE);
    document
}

fn scan_20_lines(c: &mut Criterion) {
    let document = invoice_with_lines(20);
    let mut group = c.benchmark_group("xml");
    group.throughput(Throughput::Bytes(
        u64::try_from(document.len()).unwrap_or(u64::MAX),
    ));
    group.bench_function("scan_20_lines", |b| {
        b.iter(|| scan(black_box(document.as_bytes()), Limits::default()));
    });
    group.finish();
}

fn scan_5mb(c: &mut Criterion) {
    let document = invoice_with_approx_bytes(5 * 1024 * 1024);
    let mut group = c.benchmark_group("xml_large");
    group.sample_size(10);
    group.throughput(Throughput::Bytes(
        u64::try_from(document.len()).unwrap_or(u64::MAX),
    ));
    group.bench_function("scan_5mb", |b| {
        b.iter(|| scan(black_box(document.as_bytes()), Limits::default()));
    });
    group.finish();
}

fn reader_events_20_lines(c: &mut Criterion) {
    let document = invoice_with_lines(20);
    let mut group = c.benchmark_group("xml_events");
    group.throughput(Throughput::Bytes(
        u64::try_from(document.len()).unwrap_or(u64::MAX),
    ));
    group.bench_function("reader_20_lines", |b| {
        b.iter(|| {
            let mut reader = Reader::new(black_box(document.as_bytes()), Limits::default());
            let mut events = 0usize;
            loop {
                match reader.read_event() {
                    Ok(Event::Eof) | Err(_) => break,
                    Ok(_) => events = events.saturating_add(1),
                }
            }
            black_box(events)
        });
    });
    group.finish();
}

criterion_group!(benches, scan_20_lines, scan_5mb, reader_events_20_lines);
criterion_main!(benches);
