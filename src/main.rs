use tes3::esp::*;
use rand::Rng;

use std::fs;
use std::collections::HashSet;

// Run this script in `Data Files`, along with the Zombie mod plugin.
fn main() -> std::io::Result<()> {
    // Get the names of lights that need to be removed from cells
    let mut refs_to_scale = HashSet::new();
    let mut stack = Vec::new();
    let mut rng = rand::thread_rng();

    let base_plugins = vec!["Morrowind.esm",
		   "Tribunal.esm",
		   "Bloodmoon.esm"];

    // Create the plugin
    let mut plugin_to_patch = Plugin {..Default::default()};

    // Create the plugin header
    let mut header = Header {..Default::default()};

    header.version = 1.3;

    // collect light names from the base game
    for plugin_name in &base_plugins {

	// Get the plugin size for an accurate header
	let plugin_size = fs::metadata(plugin_name)?.len();

	// Push the name and size to the header
	header.masters.push((plugin_name.to_string(), plugin_size));

	// Load the actual plugin now that clerical stuff is done
        let base_plugin = Plugin::from_path(plugin_name)?;

	// Add the refs to the existing list
	refs_to_scale.extend(collect_required_ids(&base_plugin));
    }

    plugin_to_patch.objects.push(tes3::esp::TES3Object::Header(header));

    // MastIdx should always reflect the actual master they came from
    // let mut mast_index = 1;

    for (mast_index, plugin_name) in base_plugins.iter().enumerate() {

	// Read the master
        let base_plugin = Plugin::from_path(plugin_name)?;

	// Iterate over every cell
	for cell in base_plugin.objects_of_type::<Cell>() {

	    // Ignore cells without any name at all
	    // Dat shit is in da ocean
	    if cell.name == "" {continue;}

	    let mut scaled_cell = cell.clone();

	    scaled_cell.references.clear();

	    // Check every object
	    for reference in &cell.references {
		// Match the name against the lights
		if refs_to_scale.contains(&reference.1.id.to_ascii_lowercase()) {

		    // Clone the rec for mutability
		    let mut new_ref = reference.1.clone();

		    // Random Scale
		    new_ref.scale = Some(rng.gen_range(0.35..3.0));

		    // Update the master index
		    let new_index = (mast_index as u32, reference.0.1);

		    // It seems to exist in two places
		    new_ref.mast_index = mast_index as u32;

		    // Add the new one
		    scaled_cell.references.insert(new_index, new_ref);
		}
	    }

	    if scaled_cell.references.len() > 0 {
		stack.push(tes3::esp::TES3Object::Cell(scaled_cell.clone()));
	    }
	}
	// mast_index += 1;
    }

    for cell in stack {plugin_to_patch.objects.push(cell.clone());}


    plugin_to_patch.save_path("RT3R.esp")?;

    Ok(())

}

fn collect_required_ids(plugin: &Plugin) -> HashSet<String> {
    let mut results = HashSet::new();
    for object in &plugin.objects {
        // Save the ids of any objects required by the current object.
        match object {
            TES3Object::Creature(creature) => {
                results.insert(creature.id.to_ascii_lowercase());
            },
            TES3Object::LeveledCreature(leveled_creature) => {
                results.insert(leveled_creature.id.to_ascii_lowercase());
            },
            _ => {}
        }
    }
    results
}
