//! i.MX RT 1060 chip family features.
//!
//! Use this module to customize features for the
//! 1060 chips.

pub use imxrt_iomuxc::imxrt1060 as pads;

#[path = "ccm"]
pub(crate) mod ccm {
    pub mod arm_divider;

    #[path = "pre_periph_clk_pll1.rs"]
    pub mod pre_periph_clk;

    mod periph_clk2_podf;
    mod periph_clk2_sel;

    /// Peripheral clock 2.
    pub mod periph_clk2 {
        pub use super::periph_clk2_podf::*;
        pub use super::periph_clk2_sel::*;
    }

    pub(crate) mod analog {
        pub mod pll1;
    }

    /// Re-exported by the common clock_gate module.
    pub(crate) mod clock_gate {
        use crate::chip::ccm::clock_gate;

        /// All clock gates downstream of the PERCLK root clock.
        pub const PERCLK_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::pit(),
            clock_gate::gpt_bus::<1>(),
            clock_gate::gpt_bus::<2>(),
            clock_gate::gpt_serial::<1>(),
            clock_gate::gpt_serial::<2>(),
        ];

        /// All clock gates downstream of the UART root clock.
        pub const UART_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::lpuart::<1>(),
            clock_gate::lpuart::<2>(),
            clock_gate::lpuart::<3>(),
            clock_gate::lpuart::<4>(),
            clock_gate::lpuart::<5>(),
            clock_gate::lpuart::<6>(),
            clock_gate::lpuart::<7>(),
            clock_gate::lpuart::<8>(),
        ];

        /// All clock gates downstream of the LPSPI root clock.
        pub const LPSPI_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::lpspi::<1>(),
            clock_gate::lpspi::<2>(),
            clock_gate::lpspi::<3>(),
            clock_gate::lpspi::<4>(),
        ];

        /// All clock gates downstream of the LPI2C root clock.
        pub const LPI2C_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::lpi2c::<1>(),
            clock_gate::lpi2c::<2>(),
            clock_gate::lpi2c::<3>(),
            clock_gate::lpi2c::<4>(),
        ];

        /// All SAI clock gates.
        pub const SAI_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::sai::<1>(),
            clock_gate::sai::<2>(),
            clock_gate::sai::<3>(),
        ];

        /// All clock gates downstream of the IPG root clock.
        pub const IPG_CLOCK_GATES: &[clock_gate::Locator] = &[
            clock_gate::adc::<1>(),
            clock_gate::adc::<2>(),
            clock_gate::dma(),
            clock_gate::flexpwm::<1>(),
            clock_gate::flexpwm::<2>(),
            clock_gate::flexpwm::<3>(),
            clock_gate::flexpwm::<4>(),
            // GPIOs assume that we're not "fast," since the fast
            // GPIOs run directly off the ARM / AHB clock. This
            // is safe for now, since we don't currently support fast
            // GPIOs.
            clock_gate::gpio::<1>(),
            clock_gate::gpio::<2>(),
            clock_gate::gpio::<3>(),
            clock_gate::gpio::<4>(),
            clock_gate::gpio::<5>(),
            clock_gate::trng(),
            clock_gate::snvs_lp(),
            clock_gate::snvs_hp(),
            clock_gate::usb(),
        ];
    }

    // TODO
    pub(crate) mod clko {
        /// CLKO1 output clock selections.
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]

        pub enum Clko1Selection {
            // USB1 PLL clock (divided by 2)
            Usb1PllClk = 0b0000,
            // SYS PLL clock (divided by 2)
            SysPllClk = 0b0001,
            // VIDEO PLL clock (divided by 2)
            VideoPllClk = 0b0011,
            // semc_clk_root
            SemcClk = 0101,
            // lcdif_pix_clk_root
            LcdifPixClk = 1010,
            // ahb_clk_root
            AhbClk = 1011,
            // ipg_clk_root
            IpgClk = 1100,
            // perclk_root
            Perclk = 1101,
            // ckil_sync_clk_root
            CkilSyncClk = 1110,
            // pll4_main_clk
            Pll4Main_Clk = 111,
        }

        /// CLKO2 output clock selections.
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]

        pub enum Clko2Selection {
            // USB HC root
            Usdhc1Clk = 0b00011,
            // LPI2C clock root
            Lpi2cClk = 0b00100,
            // CSI clock
            CsiClk = 0b01001,
            /// Oscillator clock root.
            OscClk = 0b01110,
            // USDHC2 clock root
            Usdhc2Clk = 0b10001,
            /// SAI1 clock root.
            Sai1Clk = 0b10010,
            /// SAI1 clock root.
            Sai2Clk = 0b10011,
            /// SAI3 clock root.
            Sai3Clk = 0b10100,
            // CAN clock root
            CanClk = 0b10111,
            /// FlexSPI clock root.
            FlexspiClk = 0b11011,
            /// UART clock root.
            UartClk = 0b11100,
            /// SPDIF0 clock root.
            Spdif0Clk = 0b11101,
        }
    }

    ccm_flexio!(
        flexio1_clk, "FLEXIO1",
        divider: (CDCDR, FLEXIO1_CLK_PODF),
        predivider: (CDCDR, FLEXIO1_CLK_PRED),
        selection: (CDCDR, FLEXIO1_CLK_SEL),
    );

    ccm_flexio!(
        flexio2_clk, "FLEXIO2",
        divider: (CS1CDR, FLEXIO2_CLK_PODF),
        predivider: (CS1CDR, FLEXIO2_CLK_PRED),
        selection: (CSCMR2, FLEXIO2_CLK_SEL),
    );
}

pub(crate) const DMA_CHANNEL_COUNT: usize = 32;
