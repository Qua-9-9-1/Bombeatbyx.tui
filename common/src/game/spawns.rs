pub fn get_pseudo_random_u32() -> u32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_nanos();
    let mut x = now as u64;
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x as u32
}

pub fn get_spawn_points(width: usize, height: usize, count: usize) -> Vec<(usize, usize)> {
    let mut candidates = vec![
        (1, 1),
        (width - 2, 1),
        (1, height - 2),
        (width - 2, height - 2),
        (width / 2, 1),
        (width / 2, height - 2),
        (1, height / 2),
        (width - 2, height / 2),
    ];

    candidates.dedup();

    let count = count.min(candidates.len()).max(1);
    if count == candidates.len() {
        let mut result = candidates;
        let mut seed = get_pseudo_random_u32();
        let n = result.len();
        for i in (1..n).rev() {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (seed as usize) % (i + 1);
            result.swap(i, j);
        }
        return result;
    }

    let mut best_subsets = Vec::new();
    let mut max_min_dist_sq = -1.0;

    let n = candidates.len();
    for mask in 0..(1 << n) {
        if (mask as i32).count_ones() as usize == count {
            let mut subset = Vec::new();
            for i in 0..n {
                if (mask & (1 << i)) != 0 {
                    subset.push(candidates[i]);
                }
            }

            let mut min_dist_sq = f64::MAX;
            for i in 0..subset.len() {
                for j in (i + 1)..subset.len() {
                    let dx = subset[i].0 as f64 - subset[j].0 as f64;
                    let dy = subset[i].1 as f64 - subset[j].1 as f64;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                    }
                }
            }

            if min_dist_sq > max_min_dist_sq {
                max_min_dist_sq = min_dist_sq;
                best_subsets.clear();
                best_subsets.push(subset);
            } else if (min_dist_sq - max_min_dist_sq).abs() < 1e-5 {
                best_subsets.push(subset);
            }
        }
    }

    let mut seed = get_pseudo_random_u32();
    let chosen_idx = (seed as usize) % best_subsets.len();
    let mut selected_subset = best_subsets[chosen_idx].clone();

    let n = selected_subset.len();
    for i in (1..n).rev() {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let j = (seed as usize) % (i + 1);
        selected_subset.swap(i, j);
    }

    selected_subset
}
