use std::{path::PathBuf, collections::HashSet};
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
        .filter_level(log::LevelFilter::Debug)
        .init();
    let args = Args::parse();
    let input = args.input;
    let mut ultrabubbles = find_ultrabubbles(&input.into())?;
    ultrabubbles.sort();
    // probably we need a multiset or don't flatten
    let ultrabubble_nodes = ultrabubbles
        .iter()
        .flat_map(|&(a, b)| [a,b])
        .collect::<HashSet<_>>();
    for node in ultrabubble_nodes {
        println!("{}", node);
    }
    for b in ultrabubbles {
        println!("{b:?}");
    }

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

    Ok(())
}
