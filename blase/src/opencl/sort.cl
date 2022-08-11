#ifndef PRECISION
    typedef float real;
    typedef uint usize;
#endif

typedef struct Block {
    usize start;
    usize end;
} block;

static inline void swap (global real* ptr, usize a, usize b) {
    const real tmp = ptr[a];
    ptr[a] = ptr[b];
    ptr[b] = tmp;
}

static inline block calculate_block (const usize n, const usize id, const usize size) {
    const usize step = n / size;
    const usize step_rem = n % size;

    usize start = id * step;
    usize end = start + step;
    
    if (id == 0) {
        start += step_rem;
    } else {
        end += step_rem;
    }

    block block = {
        start,
        end
    };

    return block;
}

static inline void quicksort (global real* ptr, const block block) {
    if (block.end - block.start < 2) return;

    const real pivot = ptr[block.end - 1];
    usize left_ptr = block.start;
    usize right_ptr = block.end - 1;

    while (true) {
        while (ptr[left_ptr] <= pivot && left_ptr < right_ptr) {
            left_ptr++;
        }

        while (ptr[right_ptr] >= pivot && left_ptr < right_ptr) {
            right_ptr--;
        }

        if (left_ptr == right_ptr) {
            swap(ptr, left_ptr, block.end - 1);

            struct Block left = { block.start, left_ptr };
            struct Block right = { left_ptr + 1, block.end };

            quicksort(ptr, left);
            quicksort(ptr, right);
            break;
        }
        
        swap(ptr, left_ptr, right_ptr);
    }
}

// Initial sorting of smaller blocks (with quicksort)
kernel void block_sort (const usize n, global real* values) {
    const block blk = calculate_block(n, get_global_id(0), get_global_size(0));
    quicksort(values, blk);
}

/// Merging of sorted block (with merge sort)
kernel void merge_blocks (const usize n, global real* values) {
    const usize id = get_global_id(0);
    usize block_size = n / get_global_size(0);

    // Get block    
    block left_blk = calculate_block(n, id, get_global_size(0));
    block right_blk = calculate_block(n, id + 1, get_global_size(0));

    // Shift positions to match block sizes
    left_blk.start *= init_block_size;
    left_blk.end *= init_block_size;

    right_blk.start *= init_block_size;
    right_blk.end = min(right_blk.end * init_block_size, n);

    printf("\n%d,%d\n%d,%d\n", 
        (int)left_blk.start, (int)left_blk.end,
        (int)right_blk.start, (int)right_blk.end
    );
}