extern crate clap;
extern crate gfa;
extern crate saboten;

// use this form for the clap 4.1 API
/*
use clap::Parser;

/// Simple program to greet a person

fn main() {
   let args = Args::parse();

   for _ in 0..args.count {
       println!("Hello {}!", args.name)
   }
}
*/

use std::{path::PathBuf, collections::HashSet};

// use clap 4.1 style as described above to parse arguments, taking --gfa as an input pangenome graph
// here we're going to load in a GFA file using the gfa crate
// we'll then decompose it into its underlying bubble structure using the saboten crate, example usage:
/*
    let mut ultrabubbles = if let Some(path) = &args.ultrabubbles_file {
        super::saboten::load_ultrabubbles(path)
    } else {
        super::saboten::find_ultrabubbles(gfa_path)
    }?;

    info!("Using {} ultrabubbles", ultrabubbles.len());

    ultrabubbles.sort();

    let ultrabubble_nodes = ultrabubbles
        .iter()
        .flat_map(|&(a, b)| {
            use std::iter::once;
            once(a).chain(once(b))
        })
        .collect::<FnvHashSet<_>>();
*/
// we'll then output the bubble structure as a GFA file and work through it with this kind of pattern
/*
    all_vcf_records.par_extend(
        ultrabubbles
            .par_iter()
            .progress_with(p_bar)
            .filter_map(|&(from, to)| {
                let vars = variants::detect_variants_in_sub_paths(
                    &var_config,
                    &path_data,
                    ref_path_names.as_ref(),
                    &path_indices,
                    from,
                    to,
                )?;

                let vcf_records = variants::variant_vcf_record(&vars);
                Some(vcf_records)
            })
            .flatten(),
    );
 */
// begin work here
use log::*;
use clap::*;
use anyhow::*;
use gfa::{parser::{GFAParser, GFAParserBuilder}, gfa::GFA};
use saboten::{*, biedgedgraph::BiedgedGraph, cactusgraph::{CactusGraph, CactusTree, BridgeForest}};

pub fn find_ultrabubbles(gfa_path: &PathBuf) -> Result<Vec<(u64, u64)>> {
    let mut parser_builder = GFAParserBuilder::all();
    parser_builder.paths = false;
    parser_builder.containments = false;
    let parser: GFAParser<usize, ()> = parser_builder.build();

    info!("Computing ultrabubbles");
    let be_graph = {
        let gfa: GFA<usize, ()> = parser.parse_file(gfa_path)?;

        debug!("Building biedged graph");
        let t = std::time::Instant::now();
        let be_graph = BiedgedGraph::from_gfa(&gfa);
        debug!(
            "  biedged graph took {:.3} ms",
            t.elapsed().as_secs_f64() * 1000.0
        );
        debug!("");

        be_graph
    };

    debug!("Building cactus graph");
    let t = std::time::Instant::now();
    let cactus_graph = CactusGraph::from_biedged_graph(&be_graph);
    debug!(
        "  cactus graph took {:.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    debug!("");

    debug!("Building cactus tree");
    let t = std::time::Instant::now();
    let cactus_tree = CactusTree::from_cactus_graph(&cactus_graph);
    debug!(
        "  cactus tree took {:.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    debug!("");

    debug!("Building bridge forest");
    let t = std::time::Instant::now();
    let bridge_forest = BridgeForest::from_cactus_graph(&cactus_graph);
    debug!(
        "  bridge forest took {:.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    debug!("");

    debug!("Finding ultrabubbles");
    let t = std::time::Instant::now();
    let ultrabubbles =
        cactusgraph::find_ultrabubbles(&cactus_tree, &bridge_forest);
    debug!(
        "  ultrabubbles took {:.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    debug!("");

    let t = std::time::Instant::now();
    let ultrabubbles = cactusgraph::inverse_map_ultrabubbles(ultrabubbles);
    debug!(
        "  inverting ultrabubbles took {:.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );

    debug!("Done computing ultrabubbles");
    Ok(ultrabubbles.into_keys().collect())
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Name of the GFA file to import
   #[arg(short, long)]
   input: String,
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    let args = Args::parse();
    let input = args.input;
    let mut ultrabubbles = find_ultrabubbles(&input.into())?;
    ultrabubbles.sort();
    let ultrabubble_nodes = ultrabubbles
        .iter()
        .flat_map(|&(a, b)| {
            use std::iter::once;
            once(a).chain(once(b))
        })
        .collect::<HashSet<_>>();
    for node in ultrabubble_nodes {
        println!("{}", node);
    }
    Ok(())
}
