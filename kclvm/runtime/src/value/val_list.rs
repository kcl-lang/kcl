//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn list_n(size: usize, val: &Self) -> Self {
        let mut list = ListValue::default();
        for _i in 0..size {
            list.values.push(val.clone());
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_bool(x: &[bool]) -> Self {
        let mut list = ListValue::default();
        for x in x.iter() {
            list.values.push(Self::bool(*x));
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_int(x: &[i64]) -> Self {
        let mut list = ListValue::default();
        for x in x.iter() {
            list.values.push(Self::int(*x));
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_float(x: &[f64]) -> Self {
        let mut list = ListValue::default();
        for x in x.iter() {
            list.values.push(Self::float(*x));
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_str(x: &[String]) -> Self {
        let mut list = ListValue::default();
        for x in x.iter() {
            list.values.push(Self::str((*x).as_ref()));
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_resize(&mut self, newsize: usize) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                if list.values.len() > newsize {
                    list.values.truncate(newsize);
                } else {
                    while list.values.len() < newsize {
                        list.values.push(Default::default());
                    }
                }
            }
            _ => panic!("Invalid list object in list_resize"),
        }
    }

    pub fn list_clear(&mut self) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                list.values.clear();
            }
            _ => panic!("Invalid list object in list_clear"),
        }
    }

    pub fn list_get(&self, i: isize) -> Option<Self> {
        match &*self.rc.borrow() {
            Value::list_value(ref list) => {
                let index = if i < 0 {
                    (i + list.values.len() as isize) as usize
                } else {
                    i as usize
                };
                if !list.values.is_empty() {
                    Some(list.values.as_slice()[index].clone())
                } else {
                    None
                }
            }
            _ => panic!("Invalid list object in list_get"),
        }
    }

    pub fn list_get_option(&self, i: isize) -> Option<Self> {
        match &*self.rc.borrow() {
            Value::list_value(ref list) => {
                let index = if i < 0 {
                    (i + list.values.len() as isize) as usize
                } else {
                    i as usize
                };
                if !list.values.is_empty() && index < list.values.len() {
                    Some(list.values.as_slice()[index].clone())
                } else {
                    None
                }
            }
            _ => panic!("Invalid list object in list_get_option"),
        }
    }

    pub fn list_set(&mut self, i: usize, v: &Self) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                if i < list.values.len() {
                    list.values.as_mut_slice()[i] = v.clone();
                }
            }
            _ => panic!("Invalid list object in list_set"),
        }
    }

    pub fn list_pop(&mut self) -> Option<Self> {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => list.values.pop(),
            _ => panic!("Invalid list object in list_pop"),
        }
    }

    pub fn list_pop_first(&mut self) -> Option<Self> {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                if !list.values.is_empty() {
                    Some(list.values.remove(0))
                } else {
                    None
                }
            }
            _ => panic!("Invalid list object in list_pop"),
        }
    }

    pub fn list_append(&mut self, v: &Self) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                list.values.push(v.clone());
            }
            _ => panic!(
                "Invalid list object in list_append {} {:?}",
                self.to_json_string(),
                v
            ),
        }
    }

    pub fn list_append_unpack(&mut self, x_or_list: &Self) {
        match &mut*self.rc.borrow_mut() {
            Value::list_value(list) => match &*x_or_list.rc.borrow() {
                Value::list_value(ref list_b) => {
                    for x in list_b.values.iter() {
                        list.values.push(x.clone());
                    }
                }
                Value::dict_value(ref dict_b) => {
                    for (x, _) in dict_b.values.iter() {
                        list.values.push(Self::str(x.as_str()));
                    }
                }
                Value::schema_value(ref schema_b) => {
                    for (x, _) in schema_b.config.values.iter() {
                        list.values.push(Self::str(x.as_str()));
                    }
                }
                Value::none | Value::undefined => { /*Do nothing on unpacking None/Undefined*/ }
                _ => panic!("only list, dict and schema object can be used with unpack operators * and **, got {x_or_list}"),
            },
            _ => panic!("Invalid list object in list_append_unpack"),
        }
    }

    pub fn list_append_unpack_first(&mut self, x_or_list: &Self) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => match &*x_or_list.rc.borrow() {
                Value::list_value(ref list_b) => {
                    for (i, x) in list_b.values.iter().enumerate() {
                        list.values.insert(i, x.clone());
                    }
                }
                Value::dict_value(ref dict_b) => {
                    for (i, x) in dict_b.values.iter().enumerate() {
                        list.values.insert(i, Self::str(x.0.as_str()));
                    }
                }
                Value::schema_value(ref schema_b) => {
                    for (i, x) in schema_b.config.values.iter().enumerate() {
                        list.values.insert(i, Self::str(x.0.as_str()));
                    }
                }
                Value::none | Value::undefined => { /*Do nothing on unpacking None/Undefined*/ }
                _ => {
                    // Panic
                    list.values.insert(0, x_or_list.clone());
                }
            },
            _ => panic!("Invalid list object in list_append_unpack_first"),
        }
    }

    pub fn list_count(&self, item: &Self) -> usize {
        let mut count: usize = 0;
        match &*self.rc.borrow() {
            Value::list_value(ref list) => {
                for v in &list.values {
                    if v == item {
                        count += 1;
                    }
                }
            }
            _ => panic!("Invalid list object in list_find"),
        }
        count
    }

    pub fn list_find(&self, item: &Self) -> isize {
        match &*self.rc.borrow() {
            Value::list_value(ref list) => {
                for (i, v) in list.values.iter().enumerate() {
                    if v == item {
                        return i as isize;
                    }
                }
            }
            _ => panic!("Invalid list object in list_find"),
        }
        -1
    }

    pub fn list_insert_at(&mut self, i: usize, v: &Self) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                list.values.insert(i, v.clone());
            }
            _ => panic!("Invalid list object in list_insert_at"),
        }
    }

    pub fn list_remove_at(&mut self, i: usize) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                list.values.remove(i);
            }
            _ => panic!("Invalid list object in list_remove_at"),
        }
    }

    pub fn list_remove(&mut self, item: &ValueRef) {
        match &mut *self.rc.borrow_mut() {
            Value::list_value(list) => {
                let mut index: Option<usize> = None;
                for (i, v) in list.values.iter().enumerate() {
                    if v == item {
                        index = Some(i);
                    }
                }
                if let Some(index) = index {
                    list.values.remove(index);
                }
            }
            _ => panic!("Invalid list object in list_remove_at"),
        }
    }

    pub fn slice_unpack(start: &ValueRef, stop: &ValueRef, step: &ValueRef) -> (i64, i64, i64) {
        let start_val;
        let step_val;
        let stop_val;
        match &*step.rc.borrow() {
            Value::int_value(ref step) => {
                step_val = *step;
                if step_val == 0 {
                    panic!("slice step cannot be zero");
                }
            }
            _ => {
                step_val = 1;
            }
        }
        match &*start.rc.borrow() {
            Value::int_value(ref start) => start_val = *start,
            _ => {
                if step_val < 0 {
                    start_val = i64::MAX;
                } else {
                    start_val = 0;
                }
            }
        }
        match &*stop.rc.borrow() {
            Value::int_value(ref stop) => stop_val = *stop,
            _ => {
                if step_val < 0 {
                    stop_val = i64::MIN;
                } else {
                    stop_val = i64::MAX;
                }
            }
        }
        (start_val, stop_val, step_val)
    }

    pub fn slice_adjust_indices(
        len: i64,
        mut start: i64,
        mut stop: i64,
        step: i64,
    ) -> (i64, i64, i64) {
        assert!(step != 0);
        if start < 0 {
            start += len;
            if start < 0 {
                if step < 0 {
                    start = -1;
                } else {
                    start = 0;
                }
            }
        } else if start >= len {
            if step < 0 {
                start = len - 1;
            } else {
                start = len;
            }
        }

        if stop < 0 {
            stop += len;
            if stop < 0 {
                if step < 0 {
                    stop = -1;
                } else {
                    stop = 0;
                }
            }
        } else if stop >= len {
            if step < 0 {
                stop = len - 1;
            } else {
                stop = len;
            }
        }
        let mut slice_len = 0;

        if step < 0 {
            if stop < start {
                slice_len = (start - stop - 1) / (-step) + 1;
            }
        } else if start < stop {
            slice_len = (stop - start - 1) / step + 1;
        }

        (start, stop, slice_len)
    }

    pub fn list_slice(&self, start: &ValueRef, stop: &ValueRef, step: &ValueRef) -> ValueRef {
        match &*self.rc.borrow() {
            Value::list_value(ref list) => {
                let (start, stop, step) = ValueRef::slice_unpack(start, stop, step);
                let (start, _stop, slice_len) =
                    ValueRef::slice_adjust_indices(list.values.len() as i64, start, stop, step);
                let mut slice = ValueRef::list(None);
                let mut cur = start;
                for _i in 1..(slice_len + 1) {
                    slice.list_append(&list.values.as_slice()[cur as usize].clone());
                    cur += step;
                }
                slice
            }
            Value::str_value(ref str) => {
                let (start, stop, step) = ValueRef::slice_unpack(start, stop, step);
                let (start, _stop, slice_len) =
                    ValueRef::slice_adjust_indices(str.chars().count() as i64, start, stop, step);
                let mut slice = String::new();
                let mut cur = start;
                for _i in 1..(slice_len + 1) {
                    let char = str.chars().nth(cur as usize).unwrap();
                    slice.push_str(&char.to_string());
                    cur += step;
                }
                ValueRef::str(&slice)
            }
            _ => panic!("invalid slice object {}", self.type_str()),
        }
    }
}

#[cfg(test)]
mod test_value_list {

    use crate::*;

    #[test]
    fn test_slice_unpack() {
        let cases = [
            (1, 1, 1, (1, 1, 1)),
            (1, 5, 1, (1, 5, 1)),
            (5, 1, -1, (5, 1, -1)),
            (-1, -5, -1, (-1, -5, -1)),
        ];
        for (start, stop, step, expected) in cases {
            let start = ValueRef::int(start);
            let stop = ValueRef::int(stop);
            let step = ValueRef::int(step);
            let result = ValueRef::slice_unpack(&start, &stop, &step);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_slice_adjust_indices() {
        let cases = [
            (1, 1, 1, 1, (1, 1, 0)),
            (3, 1, 2, 1, (1, 2, 1)),
            (5, 1, 3, 2, (1, 3, 1)),
            (5, 1, -1, 1, (1, 4, 3)),
            (5, -1, -3, -1, (4, 2, 2)),
        ];
        for (len, start, stop, step, expected) in cases {
            assert_eq!(
                ValueRef::slice_adjust_indices(len, start, stop, step),
                expected
            )
        }
    }

    #[test]
    fn test_list_slice() {
        let mut list = ValueRef::list(None);
        for i in 1..11 {
            list.list_append(&ValueRef::int(i));
        }

        /*
        slice=list[1:9:2]
        expect_slice=[2,4,6,8]
        */
        let mut slice = list.list_slice(&ValueRef::int(1), &ValueRef::int(9), &ValueRef::int(2));
        let mut expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(2),
            &ValueRef::int(4),
            &ValueRef::int(6),
            &ValueRef::int(8),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[1:5]
        expect_slice=[2,3,4,5]
        */
        slice = list.list_slice(&ValueRef::int(1), &ValueRef::int(5), &ValueRef::undefined());
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(2),
            &ValueRef::int(3),
            &ValueRef::int(4),
            &ValueRef::int(5),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[:3]
        expect_slice=[1,2,3]
        */
        slice = list.list_slice(
            &ValueRef::undefined(),
            &ValueRef::int(3),
            &ValueRef::undefined(),
        );
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(2),
            &ValueRef::int(3),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[7:]
        expect_slice=[8,9,10]
        */
        slice = list.list_slice(
            &ValueRef::int(7),
            &ValueRef::undefined(),
            &ValueRef::undefined(),
        );
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(8),
            &ValueRef::int(9),
            &ValueRef::int(10),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[7::-2]
        expect_slice=[8,6,4,2]
        */
        slice = list.list_slice(
            &ValueRef::int(7),
            &ValueRef::undefined(),
            &ValueRef::int(-2),
        );
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(8),
            &ValueRef::int(6),
            &ValueRef::int(4),
            &ValueRef::int(2),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[::-2]
        expect_slice=[10,8,6,4,2]
        */
        slice = list.list_slice(
            &ValueRef::undefined(),
            &ValueRef::undefined(),
            &ValueRef::int(-2),
        );
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(10),
            &ValueRef::int(8),
            &ValueRef::int(6),
            &ValueRef::int(4),
            &ValueRef::int(2),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[-8:-3:2]
        expect_slice=[3,5,7]
        */
        slice = list.list_slice(&ValueRef::int(-8), &ValueRef::int(-3), &ValueRef::int(2));
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(3),
            &ValueRef::int(5),
            &ValueRef::int(7),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[-3:-8:-2]
        expect_slice=[3,5,7]
        */
        slice = list.list_slice(&ValueRef::int(-8), &ValueRef::int(-3), &ValueRef::int(2));
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(3),
            &ValueRef::int(5),
            &ValueRef::int(7),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[-100:200]
        expect_slice=list
        */
        slice = list.list_slice(
            &ValueRef::int(-100),
            &ValueRef::int(200),
            &ValueRef::undefined(),
        );

        assert!(list.cmp_equal(&slice));

        /*
        slice=list[200:-100:-2]
        expect_slice=[10,8,6,4,2]
        */
        slice = list.list_slice(
            &ValueRef::int(200),
            &ValueRef::int(-100),
            &ValueRef::int(-2),
        );
        expect_slice = ValueRef::list(Some(&[
            &ValueRef::int(10),
            &ValueRef::int(8),
            &ValueRef::int(6),
            &ValueRef::int(4),
            &ValueRef::int(2),
        ]));
        assert!(expect_slice.cmp_equal(&slice));

        /*
        slice=list[8:2]
        expect_slice=[]
        */
        slice = list.list_slice(&ValueRef::int(8), &ValueRef::int(2), &ValueRef::undefined());
        expect_slice = ValueRef::list(None);
        assert!(expect_slice.cmp_equal(&slice));
    }
}
