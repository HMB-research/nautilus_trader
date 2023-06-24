// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2023 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{
    cmp,
    fmt::{Display, Formatter},
};

use nautilus_core::{correctness, time::UnixNanos};
use pyo3::{prelude::*, pyclass::CompareOp, types::PyDict};
use serde::{Deserialize, Serialize};

use crate::{
    enums::PriceType,
    identifiers::instrument_id::InstrumentId,
    types::{fixed::FIXED_PRECISION, price::Price, quantity::Quantity},
};

/// Represents a single quote tick in a financial market.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[pyclass]
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
    pub bid: Price,
    pub ask: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}

impl QuoteTick {
    #[must_use]
    pub fn new(
        instrument_id: InstrumentId,
        bid: Price,
        ask: Price,
        bid_size: Quantity,
        ask_size: Quantity,
        ts_event: UnixNanos,
        ts_init: UnixNanos,
    ) -> Self {
        correctness::u8_equal(
            bid.precision,
            ask.precision,
            "bid.precision",
            "ask.precision",
        );
        correctness::u8_equal(
            bid_size.precision,
            ask_size.precision,
            "bid_size.precision",
            "ask_size.precision",
        );
        Self {
            instrument_id,
            bid,
            ask,
            bid_size,
            ask_size,
            ts_event,
            ts_init,
        }
    }

    #[must_use]
    pub fn extract_price(&self, price_type: PriceType) -> Price {
        match price_type {
            PriceType::Bid => self.bid,
            PriceType::Ask => self.ask,
            PriceType::Mid => Price::from_raw(
                (self.bid.raw + self.ask.raw) / 2,
                cmp::min(self.bid.precision + 1, FIXED_PRECISION),
            ),
            _ => panic!("Cannot extract with price type {price_type}"),
        }
    }

    /// Return JSON encoded bytes representation of the object.
    fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    /// Return MsgPack encoded bytes representation of the object.
    fn to_msgpack_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }
}

impl Display for QuoteTick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{}",
            self.instrument_id, self.bid, self.ask, self.bid_size, self.ask_size, self.ts_event,
        )
    }
}

#[pymethods]
impl QuoteTick {
    #[new]
    fn new_py(
        instrument_id: InstrumentId,
        bid_price: Price,
        ask_price: Price,
        bid_size: Quantity,
        ask_size: Quantity,
        ts_event: UnixNanos,
        ts_init: UnixNanos,
    ) -> Self {
        Self::new(
            instrument_id,
            bid_price,
            ask_price,
            bid_size,
            ask_size,
            ts_event,
            ts_init,
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => self.ne(other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[getter]
    fn instrument_id(&self) -> InstrumentId {
        self.instrument_id.clone()
    }

    #[getter]
    fn bid_price(&self) -> Price {
        self.bid
    }

    #[getter]
    fn ask_price(&self) -> Price {
        self.ask
    }

    #[getter]
    fn bid_size(&self) -> Quantity {
        self.bid_size
    }

    #[getter]
    fn ask_size(&self) -> Quantity {
        self.ask_size
    }

    #[getter]
    fn ts_event(&self) -> UnixNanos {
        self.ts_event
    }

    #[getter]
    fn ts_init(&self) -> UnixNanos {
        self.ts_init
    }

    fn extract_price_py(&self, price_type: PriceType) -> PyResult<Price> {
        Ok(self.extract_price(price_type))
    }

    /// Return a dictionary representation of the object.
    fn to_dict(&self) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);

            dict.set_item("type", stringify!(QuoteTick)).unwrap();
            dict.set_item("instrument_id", self.instrument_id.to_string())
                .unwrap();
            dict.set_item("bid", self.bid.to_string()).unwrap();
            dict.set_item("ask", self.ask.to_string()).unwrap();
            dict.set_item("bid_size", self.bid_size.to_string())
                .unwrap();
            dict.set_item("ask_size", self.ask_size.to_string())
                .unwrap();
            dict.set_item("ts_event", self.ts_event).unwrap();
            dict.set_item("ts_init", self.ts_init).unwrap();

            dict.into_py(py)
        })
    }

    /// Return JSON encoded bytes representation of the object.
    fn to_json(&self) -> Py<PyAny> {
        Python::with_gil(|py| self.to_json_bytes().into_py(py))
    }

    /// Return MsgPack encoded bytes representation of the object.
    fn to_msgpack(&self) -> Py<PyAny> {
        Python::with_gil(|py| self.to_msgpack_bytes().into_py(py))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use crate::{
        data::quote::QuoteTick,
        enums::PriceType,
        identifiers::instrument_id::InstrumentId,
        types::{price::Price, quantity::Quantity},
    };

    #[test]
    fn test_to_string() {
        let tick = QuoteTick {
            instrument_id: InstrumentId::from_str("ETHUSDT-PERP.BINANCE").unwrap(),
            bid: Price::new(10000.0, 4),
            ask: Price::new(10001.0, 4),
            bid_size: Quantity::new(1.0, 8),
            ask_size: Quantity::new(1.0, 8),
            ts_event: 0,
            ts_init: 0,
        };
        assert_eq!(
            tick.to_string(),
            "ETHUSDT-PERP.BINANCE,10000.0000,10001.0000,1.00000000,1.00000000,0"
        );
    }

    #[rstest(
        input,
        expected,
        case(PriceType::Bid, 10_000_000_000_000),
        case(PriceType::Ask, 10_001_000_000_000),
        case(PriceType::Mid, 10_000_500_000_000)
    )]
    fn test_extract_price(input: PriceType, expected: i64) {
        let tick = QuoteTick {
            instrument_id: InstrumentId::from_str("ETHUSDT-PERP.BINANCE").unwrap(),
            bid: Price::new(10000.0, 4),
            ask: Price::new(10001.0, 4),
            bid_size: Quantity::new(1.0, 8),
            ask_size: Quantity::new(1.0, 8),
            ts_event: 0,
            ts_init: 0,
        };

        let result = tick.extract_price(input).raw;
        assert_eq!(result, expected);
    }
}
