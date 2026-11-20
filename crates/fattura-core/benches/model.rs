//! Parse-cost benchmarks for the typed model (E2.2).
//!
//! The fixture is generated deterministically at run time: a minimal header
//! followed by as many 20-line bodies as needed to exceed 5 MB. This is the
//! shape that stresses the per-body collections and the repeated-line loop; the
//! 5 MB figure is the input-size limit from `P1 §9.2` (control `00003`).
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p fattura-core --bench model
//! ```

// Criterion's macros expand to an undocumented `fn benches()`; benchmarks are
// not part of the public API surface.
#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]

use std::fmt::Write as _;
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fattura_core::model::{Document, parse};

/// Target fixture size: the `5 MB` invoice limit (`P1 §9.2`, `00003`).
const TARGET_BYTES: usize = 5_000_000;

/// Number of `DettaglioLinee` per generated body.
const LINES_PER_BODY: usize = 20;

fn generated_fixture() -> Vec<u8> {
    let mut xml = String::with_capacity(TARGET_BYTES.saturating_add(4096));
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<FatturaElettronica versione="FPR12">
  <FatturaElettronicaHeader>
    <DatiTrasmissione>
      <IdTrasmittente><IdPaese>IT</IdPaese><IdCodice>01234567890</IdCodice></IdTrasmittente>
      <ProgressivoInvio>00001</ProgressivoInvio>
      <FormatoTrasmissione>FPR12</FormatoTrasmissione>
      <CodiceDestinatario>0000000</CodiceDestinatario>
    </DatiTrasmissione>
    <CedentePrestatore>
      <DatiAnagrafici>
        <IdFiscaleIVA><IdPaese>IT</IdPaese><IdCodice>01234567890</IdCodice></IdFiscaleIVA>
        <Anagrafica><Denominazione>Fornitore</Denominazione></Anagrafica>
        <RegimeFiscale>RF01</RegimeFiscale>
      </DatiAnagrafici>
      <Sede><Indirizzo>Via Roma 1</Indirizzo><CAP>00100</CAP><Comune>Roma</Comune><Nazione>IT</Nazione></Sede>
    </CedentePrestatore>
    <CessionarioCommittente>
      <DatiAnagrafici><Anagrafica><Denominazione>Cliente</Denominazione></Anagrafica></DatiAnagrafici>
      <Sede><Indirizzo>Via Milano 2</Indirizzo><CAP>20100</CAP><Comune>Milano</Comune><Nazione>IT</Nazione></Sede>
    </CessionarioCommittente>
  </FatturaElettronicaHeader>
"#,
    );

    let mut body = 0_usize;
    while xml.len() < TARGET_BYTES {
        body = body.saturating_add(1);
        let _ = write!(
            xml,
            r"  <FatturaElettronicaBody>
    <DatiGenerali>
      <DatiGeneraliDocumento>
        <TipoDocumento>TD01</TipoDocumento><Divisa>EUR</Divisa>
        <Data>2026-11-20</Data><Numero>{body}</Numero>
      </DatiGeneraliDocumento>
    </DatiGenerali>
    <DatiBeniServizi>
"
        );
        for line in 1..=LINES_PER_BODY {
            let _ = write!(
                xml,
                r"      <DettaglioLinee>
        <NumeroLinea>{line}</NumeroLinea>
        <Descrizione>Riga di prova numero {line}</Descrizione>
        <Quantita>1.00</Quantita>
        <PrezzoUnitario>100.00</PrezzoUnitario>
        <PrezzoTotale>100.00</PrezzoTotale>
        <AliquotaIVA>22.00</AliquotaIVA>
      </DettaglioLinee>
"
            );
        }
        xml.push_str(
            r"      <DatiRiepilogo>
        <AliquotaIVA>22.00</AliquotaIVA>
        <ImponibileImporto>2000.00</ImponibileImporto>
        <Imposta>440.00</Imposta>
      </DatiRiepilogo>
    </DatiBeniServizi>
  </FatturaElettronicaBody>
",
        );
    }
    xml.push_str("</FatturaElettronica>\n");
    xml.into_bytes()
}

/// A 12.9 KB invoice with every element of `P1 §6`, close to the 10 KB class
/// used by the `PLAN §2.4` latency budget.
const COMPLETE_12KB: &[u8] = include_bytes!("../tests/data/model/complete.xml");

fn model_parse(criterion: &mut Criterion) {
    criterion.bench_function("parse_complete_12kb", |bencher| {
        bencher.iter(|| {
            let document = parse(black_box(COMPLETE_12KB)).unwrap();
            black_box(document.bodies.len());
        });
    });

    let input = generated_fixture();
    let mut group = criterion.benchmark_group("model");
    group.throughput(Throughput::Bytes(
        u64::try_from(input.len()).unwrap_or(u64::MAX),
    ));
    group.sample_size(10);
    group.bench_function("parse_5mb_lotto", |bencher| {
        bencher.iter(|| {
            let document: Document<'_> = parse(black_box(input.as_slice())).unwrap();
            black_box(document.bodies.len());
        });
    });
    group.finish();
}

criterion_group!(benches, model_parse);
criterion_main!(benches);
