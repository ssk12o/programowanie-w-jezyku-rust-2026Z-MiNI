#[derive(Debug, Clone, Default)]
struct NumberWithUnit {
    unit: String,
    value: f64,
}

#[allow(dead_code)]
impl NumberWithUnit {
    /// constructor function that creates unitless numver with unit(none)
    /// ```
    /// let val = 12.0
    /// let un = NumberWithUnit::unitless(val);
    /// ```
    /// * `value`: value of value with unit
    fn unitless(value: f64) -> Self {
        Self {
            value,
            unit: String::from(""),
        }
    }

    fn with_unit(value: f64, unit: String) -> Self {
        Self { value, unit }
    }

    fn with_unit_from(other: Self, value: f64) -> Self {
        Self {
            value,
            unit: other.unit.clone(),
        }
    }

    fn is_unitless(&self) -> bool {
        self.unit.is_empty()
    }

    fn add(self, other: Self) -> Self {
        if self.unit != other.unit {
            panic!("Cannot add values of different units!");
        }
        Self {
            value: self.value + other.value,
            unit: self.unit,
        }
    }
    fn mul(self, other: Self) -> Self {
        let val = self.value * other.value;
        let compount_unit: String = match (self.is_unitless(), other.is_unitless()) {
            (true, true) => "".to_string(),
            (true, false) => other.unit,
            (false, true) => self.unit,
            (false, false) => format!("{}*{}", self.unit, other.unit),
        };

        Self {
            value: val,
            unit: compount_unit,
        }
    }
    fn div(self, other: Self) -> Self {
        let val = self.value / other.value;
        let compount_unit: String = match (self.is_unitless(), other.is_unitless()) {
            (true, true) => "".to_string(),
            (true, false) => format!("1/{}", other.unit),
            (false, true) => self.unit,
            (false, false) => format!("{}/{}", self.unit, other.unit),
        };

        Self {
            value: val,
            unit: compount_unit,
        }
    }

    fn add_in_place(&mut self, other: &Self) {
        if self.unit != other.unit {
            panic!("Cannot add values of different units!");
        }
        self.value += other.value;
    }
    fn mul_in_place(&mut self, other: &Self) {
        self.value *= other.value;

        self.unit = NumberWithUnit::calculate_unit(self, other, "*");
    }
    fn div_in_place(&mut self, other: &Self) {
        self.value /= other.value;

        self.unit = NumberWithUnit::calculate_unit(self, other, "/")
    }

    fn calculate_unit(nwu1: &NumberWithUnit, nwu2: &NumberWithUnit, action: &str) -> String {
        match (nwu1.is_unitless(), nwu2.is_unitless()) {
            (true, true) => "".to_string(),
            (true, false) => nwu2.unit.clone(),
            (false, true) => nwu1.unit.clone(),
            (false, false) => format!("{}{action}{}", nwu1.unit, nwu2.unit),
        }
    }
}

fn mul_vals(vals: &[NumberWithUnit]) -> NumberWithUnit {
    let mut mult = NumberWithUnit::unitless(1.0);

    for sl in vals {
        mult.mul_in_place(sl);
    }

    mult
}
fn mul_vals_vec(vec: Vec<NumberWithUnit>) -> NumberWithUnit {
    let mut mult = NumberWithUnit::unitless(1.0);

    for sl in &vec {
        mult.mul_in_place(sl);
    }

    mult
}

struct DoubleString(String, String);

impl DoubleString {
    fn from_strs(str_1: &str, str_2: &str) -> Self {
        Self(str_1.to_string(), str_2.to_string())
    }

    #[allow(clippy::ptr_arg)]
    fn from_strings(str_1: &String, str_2: &String) -> Self {
        Self(str_1.clone(), str_2.clone())
    }

    fn show(&self) {
        println!("({}, {})", self.0, self.1);
    }
}

fn main() {
    let un_unitless = NumberWithUnit::unitless(12.0);
    let un_withunit = NumberWithUnit::with_unit(12.0, "m".to_string());
    let un_fromanother = NumberWithUnit::with_unit_from(un_withunit.clone(), 120.0);

    println!("{un_unitless:?}");
    println!("{un_withunit:?}");
    println!("{un_fromanother:?}");
    println!(
        "{}{}",
        un_unitless.is_unitless(),
        un_fromanother.is_unitless()
    );

    let distance = NumberWithUnit::with_unit(10.0, "m".to_string());
    let time = NumberWithUnit::with_unit(10.0, "s".to_string());

    let speed = distance.clone().div(time.clone());
    println!("distance is {distance:?}, time is {time:?}, so speed is {speed:?}.");

    let table_of_vals = [
        NumberWithUnit::unitless(1.0),
        distance.clone(),
        speed.clone(),
    ];
    let _vals_1 = mul_vals(&table_of_vals);
    let _vals_2 = mul_vals(&table_of_vals);

    let vec_of_vals = vec![
        NumberWithUnit::unitless(1.0),
        distance.clone(),
        speed.clone(),
    ];
    // let vecs1 = mul_vals_vec(vec_of_vals);
    let _vecs1 = mul_vals_vec(vec_of_vals.clone());
    let _vecs2 = mul_vals_vec(vec_of_vals);

    let my_string = "meow".to_string();
    let my_slice = "woem";

    let ds1 = DoubleString::from_strs(&my_string, my_slice);
    let ds2 = DoubleString::from_strings(&my_string, &my_slice.to_string());

    ds1.show();
    ds2.show();
}
