mod common;

use anyhow::Result;
use common::{
    RESULT_ZP_ADDR, require_color_diversity, run_rom_frames, run_rom_status, run_rom_tv_sha1,
    run_rom_zeropage_result,
};
use ctor::ctor;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const DEFAULT_FRAMES: usize = 1800;
// instr_test v3 needs a bit longer than the default to complete all 16 subtests.
const INSTR_TEST_V3_FRAMES: usize = 2500;
// instr_test v5 needs a bit longer than the default to complete all 16 subtests.
const INSTR_TEST_V5_FRAMES: usize = 2500;

#[ctor]
fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

#[test]
fn _240pee_suite() -> Result<()> {
    for rom in ["240pee/240pee-bnrom.nes", "240pee/240pee.nes"] {
        run_rom_frames(rom, 300, |nes| require_color_diversity(nes, 4))?;
    }
    Ok(())
}

#[test]
fn mmc1_a12_suite() -> Result<()> {
    run_rom_frames("MMC1_A12/mmc1_a12.nes", 600, |nes| {
        require_color_diversity(nes, 4)
    })
}

/// Interactive paddle controller test ROM.
/// See `vendor/nes-test-roms/PaddleTest3/Info.txt` for usage; this ROM
/// does not expose a $6000 status byte protocol, so it must be verified
/// manually by running it in an emulator and following the on-screen
/// instructions.
#[test]
#[ignore = "interactive ROM; requires manual verification per PaddleTest3/Info.txt"]
fn paddletest3_manual() -> Result<()> {
    run_rom_frames("PaddleTest3/PaddleTest.nes", 300, |_| Ok(()))
}

#[test]
fn apu_mixer_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "apu_mixer/dmc.nes",
        "apu_mixer/noise.nes",
        "apu_mixer/square.nes",
        "apu_mixer/triangle.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn apu_reset_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "apu_reset/4015_cleared.nes",
        "apu_reset/4017_timing.nes",
        "apu_reset/4017_written.nes",
        "apu_reset/irq_flag_cleared.nes",
        "apu_reset/len_ctrs_enabled.nes",
        "apu_reset/works_immediately.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn apu_test_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "apu_test/apu_test.nes",
        "apu_test/rom_singles/1-len_ctr.nes",
        "apu_test/rom_singles/2-len_table.nes",
        "apu_test/rom_singles/3-irq_flag.nes",
        "apu_test/rom_singles/4-jitter.nes",
        "apu_test/rom_singles/5-len_timing.nes",
        "apu_test/rom_singles/6-irq_flag_timing.nes",
        "apu_test/rom_singles/7-dmc_basics.nes",
        "apu_test/rom_singles/8-dmc_rates.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn blargg_apu_2005_07_30_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "blargg_apu_2005.07.30/01.len_ctr.nes",
        "blargg_apu_2005.07.30/02.len_table.nes",
        "blargg_apu_2005.07.30/03.irq_flag.nes",
        "blargg_apu_2005.07.30/04.clock_jitter.nes",
        "blargg_apu_2005.07.30/05.len_timing_mode0.nes",
        "blargg_apu_2005.07.30/06.len_timing_mode1.nes",
        "blargg_apu_2005.07.30/07.irq_flag_timing.nes",
        "blargg_apu_2005.07.30/08.irq_timing.nes",
        "blargg_apu_2005.07.30/09.reset_timing.nes",
        "blargg_apu_2005.07.30/10.len_halt_timing.nes",
        "blargg_apu_2005.07.30/11.len_reload_timing.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn blargg_litewall_suite() -> Result<()> {
    for rom in [
        "blargg_litewall/blargg_litewall-10c.nes",
        "blargg_litewall/blargg_litewall-9.nes",
        "blargg_litewall/litewall2.nes",
        "blargg_litewall/litewall3.nes",
        "blargg_litewall/litewall5.nes",
    ] {
        run_rom_frames(rom, 300, |nes| require_color_diversity(nes, 8))?;
    }
    Ok(())
}

#[test]
fn blargg_nes_cpu_test5_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "blargg_nes_cpu_test5/cpu.nes",
        "blargg_nes_cpu_test5/official.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn blargg_ppu_tests_2005_09_15b_suite() -> Result<()> {
    // These ROMs report their result via on-screen text + beeps and don't expose
    // the blargg $6000 status byte protocol. They do keep the current/final
    // result code in zero-page $00F0 (used to drive the beeps), so validate
    // pass/fail via that byte instead of hashing video output.
    const RESULT_ZP: u16 = 0x00F0;
    for rom in [
        "blargg_ppu_tests_2005.09.15b/palette_ram.nes",
        "blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
        "blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
        "blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
        "blargg_ppu_tests_2005.09.15b/vram_access.nes",
    ] {
        run_rom_zeropage_result(rom, DEFAULT_FRAMES, RESULT_ZP, 0x01)?;
    }
    Ok(())
}

#[test]
fn branch_timing_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs (report result via zero-page $00F8)
    const BRANCH_FRAMES: usize = 4000;
    for rom in [
        "branch_timing_tests/1.Branch_Basics.nes",
        "branch_timing_tests/2.Backward_Branch.nes",
        "branch_timing_tests/3.Forward_Branch.nes",
    ] {
        run_rom_zeropage_result(rom, BRANCH_FRAMES, RESULT_ZP_ADDR, 0x01)?;
    }
    Ok(())
}

#[test]
fn cpu_dummy_reads_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "cpu_dummy_reads/cpu_dummy_reads.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn cpu_dummy_writes_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "cpu_dummy_writes/cpu_dummy_writes_oam.nes",
        "cpu_dummy_writes/cpu_dummy_writes_ppumem.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn cpu_exec_space_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "cpu_exec_space/test_cpu_exec_space_apu.nes",
        "cpu_exec_space/test_cpu_exec_space_ppuio.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn cpu_interrupts_v2_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "cpu_interrupts_v2/cpu_interrupts.nes",
        "cpu_interrupts_v2/rom_singles/1-cli_latency.nes",
        "cpu_interrupts_v2/rom_singles/2-nmi_and_brk.nes",
        "cpu_interrupts_v2/rom_singles/3-nmi_and_irq.nes",
        "cpu_interrupts_v2/rom_singles/4-irq_and_dma.nes",
        "cpu_interrupts_v2/rom_singles/5-branch_delays_irq.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn cpu_reset_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in ["cpu_reset/ram_after_reset.nes", "cpu_reset/registers.nes"] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn cpu_timing_test6_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "cpu_timing_test6/cpu_timing_test.nes";
        run_rom_tv_sha1(rom, Some("No inputs -- official only"))?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn dmc_dma_during_read4_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "dmc_dma_during_read4/dma_2007_read.nes",
        "dmc_dma_during_read4/dma_2007_write.nes",
        "dmc_dma_during_read4/dma_4016_read.nes",
        "dmc_dma_during_read4/double_2007_read.nes",
        "dmc_dma_during_read4/read_write_2007.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn dmc_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "dmc_tests/buffer_retained.nes",
        "dmc_tests/latency.nes",
        "dmc_tests/status.nes",
        "dmc_tests/status_irq.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn dpcmletterbox_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "dpcmletterbox/dpcmletterbox.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn exram_suite() -> Result<()> {
    run_rom_frames("exram/mmc5exram.nes", 600, |nes| {
        // Heuristic: program should execute code from MMC5 ExRAM and render
        // copper bars; ensure we actually drew a varied frame.
        require_color_diversity(nes, 8)
    })
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn full_palette_suite() -> Result<()> {
    for rom in [
        "full_palette/flowing_palette.nes",
        "full_palette/full_palette.nes",
        "full_palette/full_palette_smooth.nes",
    ] {
        run_rom_frames(rom, 120, |nes| require_color_diversity(nes, 32))?;
    }
    Ok(())
}

#[test]
fn instr_misc_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "instr_misc/instr_misc.nes",
        "instr_misc/rom_singles/01-abs_x_wrap.nes",
        "instr_misc/rom_singles/02-branch_wrap.nes",
        "instr_misc/rom_singles/03-dummy_reads.nes",
        "instr_misc/rom_singles/04-dummy_reads_apu.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn instr_test_v3_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "instr_test-v3/all_instrs.nes",
        "instr_test-v3/official_only.nes",
        "instr_test-v3/rom_singles/01-implied.nes",
        "instr_test-v3/rom_singles/02-immediate.nes",
        "instr_test-v3/rom_singles/03-zero_page.nes",
        "instr_test-v3/rom_singles/04-zp_xy.nes",
        "instr_test-v3/rom_singles/05-absolute.nes",
        "instr_test-v3/rom_singles/06-abs_xy.nes",
        "instr_test-v3/rom_singles/07-ind_x.nes",
        "instr_test-v3/rom_singles/08-ind_y.nes",
        "instr_test-v3/rom_singles/09-branches.nes",
        "instr_test-v3/rom_singles/10-stack.nes",
        "instr_test-v3/rom_singles/11-jmp_jsr.nes",
        "instr_test-v3/rom_singles/12-rts.nes",
        "instr_test-v3/rom_singles/13-rti.nes",
        "instr_test-v3/rom_singles/14-brk.nes",
        "instr_test-v3/rom_singles/15-special.nes",
    ] {
        run_rom_status(rom, INSTR_TEST_V3_FRAMES)?;
    }
    Ok(())
}

#[test]
fn instr_test_v5_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "instr_test-v5/all_instrs.nes",
        "instr_test-v5/official_only.nes",
        "instr_test-v5/rom_singles/01-basics.nes",
        "instr_test-v5/rom_singles/02-implied.nes",
        "instr_test-v5/rom_singles/03-immediate.nes",
        "instr_test-v5/rom_singles/04-zero_page.nes",
        "instr_test-v5/rom_singles/05-zp_xy.nes",
        "instr_test-v5/rom_singles/06-absolute.nes",
        "instr_test-v5/rom_singles/07-abs_xy.nes",
        "instr_test-v5/rom_singles/08-ind_x.nes",
        "instr_test-v5/rom_singles/09-ind_y.nes",
        "instr_test-v5/rom_singles/10-branches.nes",
        "instr_test-v5/rom_singles/11-stack.nes",
        "instr_test-v5/rom_singles/12-jmp_jsr.nes",
        "instr_test-v5/rom_singles/13-rts.nes",
        "instr_test-v5/rom_singles/14-rti.nes",
        "instr_test-v5/rom_singles/15-brk.nes",
        "instr_test-v5/rom_singles/16-special.nes",
    ] {
        run_rom_status(rom, INSTR_TEST_V5_FRAMES)?;
    }
    Ok(())
}

#[test]
fn instr_timing_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "instr_timing/instr_timing.nes",
        "instr_timing/rom_singles/1-instr_timing.nes",
        "instr_timing/rom_singles/2-branch_timing.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn m22chrbankingtest_suite() -> Result<()> {
    run_rom_frames("m22chrbankingtest/0-127.nes", 600, |nes| {
        require_color_diversity(nes, 4)
    })
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn mmc3_irq_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "mmc3_irq_tests/1.Clocking.nes",
        "mmc3_irq_tests/2.Details.nes",
        "mmc3_irq_tests/3.A12_clocking.nes",
        "mmc3_irq_tests/4.Scanline_timing.nes",
        "mmc3_irq_tests/5.MMC3_rev_A.nes",
        "mmc3_irq_tests/6.MMC3_rev_B.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn mmc3_test_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "mmc3_test/1-clocking.nes",
        "mmc3_test/2-details.nes",
        "mmc3_test/3-A12_clocking.nes",
        "mmc3_test/4-scanline_timing.nes",
        "mmc3_test/5-MMC3.nes",
        "mmc3_test/6-MMC6.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn mmc3_test_2_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "mmc3_test_2/rom_singles/1-clocking.nes",
        "mmc3_test_2/rom_singles/2-details.nes",
        "mmc3_test_2/rom_singles/3-A12_clocking.nes",
        "mmc3_test_2/rom_singles/4-scanline_timing.nes",
        "mmc3_test_2/rom_singles/5-MMC3.nes",
        "mmc3_test_2/rom_singles/6-MMC3_alt.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn mmc5test_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "mmc5test/mmc5test.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn mmc5test_v2_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "mmc5test_v2/mmc5test.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn nes15_1_0_0_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in ["nes15-1.0.0/nes15-NTSC.nes", "nes15-1.0.0/nes15-PAL.nes"] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn nes_instr_test_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "nes_instr_test/rom_singles/01-implied.nes",
        "nes_instr_test/rom_singles/02-immediate.nes",
        "nes_instr_test/rom_singles/03-zero_page.nes",
        "nes_instr_test/rom_singles/04-zp_xy.nes",
        "nes_instr_test/rom_singles/05-absolute.nes",
        "nes_instr_test/rom_singles/06-abs_xy.nes",
        "nes_instr_test/rom_singles/07-ind_x.nes",
        "nes_instr_test/rom_singles/08-ind_y.nes",
        "nes_instr_test/rom_singles/09-branches.nes",
        "nes_instr_test/rom_singles/10-stack.nes",
        "nes_instr_test/rom_singles/11-special.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn nmi_sync_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in ["nmi_sync/demo_ntsc.nes", "nmi_sync/demo_pal.nes"] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn nrom368_suite() -> Result<()> {
    for rom in ["nrom368/fail368.nes", "nrom368/test1.nes"] {
        run_rom_frames(rom, 600, |nes| require_color_diversity(nes, 4))?;
    }
    Ok(())
}

#[test]
fn ny2011_suite() -> Result<()> {
    run_rom_frames("ny2011/ny2011.nes", 600, |nes| {
        require_color_diversity(nes, 4)
    })
}

#[test]
fn oam_read_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "oam_read/oam_read.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn oam_stress_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "oam_stress/oam_stress.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn other_suite() -> Result<()> {
    for rom in [
        "other/2003-test.nes",
        "other/8bitpeoples_-_deadline_console_invitro.nes",
        "other/BladeBuster.nes",
        "other/Duelito.nes",
        "other/PCM.demo.wgraphics.nes",
        "other/SimpleParallaxDemo.nes",
        "other/Streemerz_bundle.nes",
        "other/apocalypse.nes",
        "other/blargg_litewall-2.nes",
        "other/blargg_litewall-9.nes",
        "other/demo jitter.nes",
        "other/demo.nes",
        "other/fceuxd.nes",
        "other/firefly.nes",
        "other/high-hopes.nes",
        "other/logo (E).nes",
        "other/manhole.nes",
        "other/max-300.nes",
        "other/midscanline.nes",
        "other/minipack.nes",
        "other/nescafe.nes",
        "other/nestest.nes",
        "other/nestopia.nes",
        "other/new-game.nes",
        "other/nintendulator.nes",
        "other/oam3.nes",
        "other/oc.nes",
        "other/physics.0.1.nes",
        "other/pulsar.nes",
        "other/quantum_disco_brothers_by_wAMMA.nes",
        "other/rastesam4.nes",
        "other/read2004.nes",
        "other/snow.nes",
        "other/test001.nes",
        "other/test28.nes",
        "other/window2_ntsc.nes",
        "other/window2_pal.nes",
        "other/window_old_ntsc.nes",
        "other/window_old_pal.nes",
    ] {
        run_rom_frames(rom, 240, |nes| require_color_diversity(nes, 4))?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn pal_apu_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "pal_apu_tests/01.len_ctr.nes",
        "pal_apu_tests/02.len_table.nes",
        "pal_apu_tests/03.irq_flag.nes",
        "pal_apu_tests/04.clock_jitter.nes",
        "pal_apu_tests/05.len_timing_mode0.nes",
        "pal_apu_tests/06.len_timing_mode1.nes",
        "pal_apu_tests/07.irq_flag_timing.nes",
        "pal_apu_tests/08.irq_timing.nes",
        "pal_apu_tests/10.len_halt_timing.nes",
        "pal_apu_tests/11.len_reload_timing.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn ppu_open_bus_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "ppu_open_bus/ppu_open_bus.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn ppu_read_buffer_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "ppu_read_buffer/test_ppu_read_buffer.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn ppu_vbl_nmi_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "ppu_vbl_nmi/ppu_vbl_nmi.nes",
        "ppu_vbl_nmi/rom_singles/01-vbl_basics.nes",
        "ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes",
        "ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes",
        "ppu_vbl_nmi/rom_singles/04-nmi_control.nes",
        "ppu_vbl_nmi/rom_singles/05-nmi_timing.nes",
        "ppu_vbl_nmi/rom_singles/06-suppression.nes",
        "ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes",
        "ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes",
        "ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes",
        "ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn read_joy3_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "read_joy3/count_errors.nes",
        "read_joy3/count_errors_fast.nes",
        "read_joy3/test_buttons.nes",
        "read_joy3/thorough_test.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn scanline_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "scanline/scanline.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn scanline_a1_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "scanline-a1/scanline.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn scrolltest_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "scrolltest/scroll.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn sprdma_and_dmc_dma_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "sprdma_and_dmc_dma/sprdma_and_dmc_dma.nes",
        "sprdma_and_dmc_dma/sprdma_and_dmc_dma_512.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn sprite_hit_tests_2005_10_05_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    //
    // These ROMs do not use the blargg $6000 status protocol. They store the
    // final result code in zero-page $00F8 (see `source/runtime/validation.a`).
    for rom in [
        "sprite_hit_tests_2005.10.05/01.basics.nes",
        "sprite_hit_tests_2005.10.05/02.alignment.nes",
        "sprite_hit_tests_2005.10.05/03.corners.nes",
        "sprite_hit_tests_2005.10.05/04.flip.nes",
        "sprite_hit_tests_2005.10.05/05.left_clip.nes",
        "sprite_hit_tests_2005.10.05/06.right_edge.nes",
        "sprite_hit_tests_2005.10.05/07.screen_bottom.nes",
        "sprite_hit_tests_2005.10.05/08.double_height.nes",
        "sprite_hit_tests_2005.10.05/09.timing_basics.nes",
        "sprite_hit_tests_2005.10.05/10.timing_order.nes",
        "sprite_hit_tests_2005.10.05/11.edge_timing.nes",
    ] {
        run_rom_zeropage_result(rom, DEFAULT_FRAMES, RESULT_ZP_ADDR, 0x01)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn sprite_overflow_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "sprite_overflow_tests/1.Basics.nes",
        "sprite_overflow_tests/2.Details.nes",
        "sprite_overflow_tests/3.Timing.nes",
        "sprite_overflow_tests/4.Obscure.nes",
        "sprite_overflow_tests/5.Emulator.nes",
    ] {
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn spritecans_2011_suite() -> Result<()> {
    run_rom_frames("spritecans-2011/spritecans.nes", 240, |nes| {
        require_color_diversity(nes, 4)
    })
}

#[test]
fn stomper_suite() -> Result<()> {
    run_rom_frames("stomper/smwstomp.nes", 300, |nes| {
        require_color_diversity(nes, 4)
    })
}

#[test]
fn tutor_suite() -> Result<()> {
    run_rom_frames("tutor/tutor.nes", 300, |nes| {
        require_color_diversity(nes, 4)
    })
}

/// TV characteristics test ROM (NTSC chroma/luma crosstalk, pixel aspect ratio).
/// See `vendor/nes-test-roms/tvpassfail/README.txt`. This ROM is meant to be
/// evaluated visually by switching screens with the controller; it does not
/// follow the $6000 status protocol, so automated pass/fail is not defined.
#[test]
#[ignore = "interactive ROM; requires manual visual verification per tvpassfail/README.txt"]
fn tvpassfail_manual() -> Result<()> {
    run_rom_frames("tvpassfail/tv.nes", DEFAULT_FRAMES, |_| Ok(()))
}

/// Vaus (Arkanoid paddle) controller test ROM.
/// See `vendor/nes-test-roms/vaus-test/README.txt`. This ROM is controlled via
/// pad/paddle input and evaluated interactively; there is no $6000 status
/// handshake, so correctness must be judged manually.
#[test]
#[ignore = "interactive ROM; requires manual verification per vaus-test/README.txt"]
fn vaus_test_manual() -> Result<()> {
    run_rom_frames("vaus-test/vaus-test.nes", DEFAULT_FRAMES, |_| Ok(()))
}

#[test]
fn vbl_nmi_timing_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    for rom in [
        "vbl_nmi_timing/1.frame_basics.nes",
        "vbl_nmi_timing/2.vbl_timing.nes",
        "vbl_nmi_timing/3.even_odd_frames.nes",
        "vbl_nmi_timing/4.vbl_clear_timing.nes",
        "vbl_nmi_timing/5.nmi_suppression.nes",
        "vbl_nmi_timing/6.nmi_disable.nes",
        "vbl_nmi_timing/7.nmi_timing.nes",
    ] {
        run_rom_zeropage_result(rom, DEFAULT_FRAMES, RESULT_ZP_ADDR, 0x01)?;
    }
    Ok(())
}

#[test]
#[ignore = "this test fails and needs investigation"]
fn volume_tests_suite() -> Result<()> {
    // TASVideos accuracy-required ROMs
    {
        let rom = "volume_tests/volumes.nes";
        run_rom_status(rom, DEFAULT_FRAMES)?;
    }
    Ok(())
}

#[test]
fn window5_suite() -> Result<()> {
    for rom in ["window5/colorwin_ntsc.nes", "window5/colorwin_pal.nes"] {
        run_rom_frames(rom, 300, |nes| require_color_diversity(nes, 4))?;
    }
    Ok(())
}
