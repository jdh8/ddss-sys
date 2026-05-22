use parking_lot::Mutex;
use std::sync::LazyLock;

static THREAD_POOL: LazyLock<Mutex<()>> = LazyLock::new(|| {
    unsafe { crate::SetMaxThreads(0) };
    Mutex::new(())
});

#[allow(clippy::cast_possible_wrap)]
const SUCCESS: i32 = crate::RETURN_NO_FAULT as i32;

/// Bit mask for an AKQJT98765432 holding in DDS's per-suit encoding
/// (rank `r` is bit `r`, so a 13-card suit occupies bits 2..=14).
const FULL_SUIT: core::ffi::c_uint = ((1 << 13) - 1) << 2;

/// Four 13-card straight flushes: N=clubs, E=diamonds, S=hearts, W=spades.
const FOUR_SF_DEAL: crate::ddTableDeal = crate::ddTableDeal {
    cards: [
        [0, 0, 0, FULL_SUIT], // N
        [0, 0, FULL_SUIT, 0], // E
        [0, FULL_SUIT, 0, 0], // S
        [FULL_SUIT, 0, 0, 0], // W
    ],
};

/// `resTable[denom][hand]`: denom order S, H, D, C, NT; hand order N, E, S, W.
const FOUR_SF_TRICKS: crate::ddTableResults = crate::ddTableResults {
    resTable: [
        [0, 13, 0, 13], // S — only diamonds/clubs holders score
        [13, 0, 13, 0], // H
        [0, 13, 0, 13], // D
        [13, 0, 13, 0], // C
        [0, 0, 0, 0],   // NT
    ],
};

#[test]
fn calc_dd_table_four_straight_flushes() {
    let mut tricks = crate::ddTableResults::default();
    let status = {
        let _guard = THREAD_POOL.lock();
        unsafe { crate::CalcDDtable(FOUR_SF_DEAL, &raw mut tricks) }
    };
    assert_eq!(status, SUCCESS);
    assert_eq!(tricks, FOUR_SF_TRICKS);
}

/// Smoke test for the fork-specific `CalcAllTablesPBNx`, the dynamic batch API.
/// Runs it with a single-deal batch in PBN form and cross-checks the result
/// against the legacy `CalcDDtable` answer.
#[test]
fn calc_all_tables_pbnx_one_deal() {
    // PBN deal string: "N:<N hand> <E hand> <S hand> <W hand>", each hand as
    // "spades.hearts.diamonds.clubs". Our deal: N=clubs, E=diamonds, S=hearts,
    // W=spades. cards[80] is null-terminated.
    const PBN: &[u8] = b"N:...AKQJT98765432 ..AKQJT98765432. .AKQJT98765432.. AKQJT98765432...\0";

    let mut deal = crate::ddTableDealPBN::default();
    assert!(PBN.len() <= deal.cards.len());
    for (dst, &b) in deal.cards.iter_mut().zip(PBN.iter()) {
        *dst = b as _;
    }

    let mut deals = [deal];
    let mut trump_filter = [0; crate::DDS_STRAINS as usize];
    let mut results = [crate::ddTableResults::default(); 1];

    let status = {
        let _guard = THREAD_POOL.lock();
        unsafe {
            crate::CalcAllTablesPBNx(
                1,
                deals.as_mut_ptr(),
                -1, // no par
                trump_filter.as_mut_ptr(),
                results.as_mut_ptr(),
                core::ptr::null_mut(),
            )
        }
    };
    assert_eq!(status, SUCCESS);
    assert_eq!(results[0], FOUR_SF_TRICKS);
}
