use lodtree::coords::OctVec;
use lodtree::*;
use rayon::prelude::*;

struct Chunk {
    // data to store in the chunk, for EG storing voxel or heighmap data
    // if you don't use editing, storing this data isn't needed, and only storing the mesh would be enough
    data: [f32; 4096], // this amount of data actually makes it slower. To see the true octree speed, replace this with [f32; 0]
}

impl Chunk {
    // this does a cheap init so it can safely be put inside the vec
    fn new(_position: OctVec) -> Self {
        Self { data: [0.0; 4096] }
    }

    // pretend this inits the data with some expensive procedural generation
    fn expensive_init(&mut self, _position: OctVec) {
        self.data = [1.0; 4096];

        // emulate a 1ms time to do things
        // we can't use sleep because thats 15ms on windows min
        let start = std::time::Instant::now();

        while start.elapsed() < std::time::Duration::from_millis(1) {}
    }

    // and pretend this makes chunks visible/invisible
    fn set_visible(&mut self, _visibility: bool) {}

    // and pretend this drops anything for when a chunk is permanently deleted
    fn cleanup(&mut self) {}
}

fn main() {
    // create an octree
    let mut tree = Tree::<Chunk, OctVec>::new(512);

    // the game loop that runs for 42 iterations
    for _ in 0..42 {
        let start_time = std::time::Instant::now();

        // get the pending updates
        if tree.prepare_update(
            &[OctVec::new(4096, 4096, 4096, 32)], // target position in the tree
            2,                                    // the amount of detail
            |position_in_tree| Chunk::new(position_in_tree), // and how we should make a new tree inside the function here. This should be done quickly
        ) {
            let duration = start_time.elapsed().as_micros();

            println!(
                "Took {} microseconds to get the tree update ready",
                duration
            );

            // if there was an update, we need to first generate new chunks with expensive_init
            tree.get_chunks_to_add_slice_mut().par_iter_mut().for_each(
                |ToAddContainer { position, chunk }| {
                    // and run expensive init
                    chunk.expensive_init(*position);
                },
            );

            // and make all chunks visible or not
            for chunk in tree.iter_chunks_to_activate_mut() {
                chunk.set_visible(true);
            }

            for chunk in tree.iter_chunks_to_deactivate_mut() {
                chunk.set_visible(false);
            }

            let start_time = std::time::Instant::now();

            // and don't forget to actually run the update
            tree.do_update();

            // now we probably want to truly clean up the chunks that are going to be deleted from memory
            for chunk in tree.iter_chunks_to_delete_mut() {
                chunk.cleanup();
            }

            // and actually clean them up
            tree.complete_update();

            let duration = start_time.elapsed().as_micros();

            println!("Took {} microseconds to execute the update", duration);
        }

        let duration = start_time.elapsed().as_micros();

        println!("Took {} microseconds to do the entire update", duration);

        // and print some data about the run
        println!("Num chunks in the tree: {}", tree.get_num_chunks());
    }
}
