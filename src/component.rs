use std::ffi::CString;

use super::IcalProperty;

pub trait IcalComponent {
    fn get_ptr(&self) -> *mut ical::icalcomponent;
    fn as_component(&self) -> &dyn IcalComponent;

    fn get_property(&self, property_kind: ical::icalproperty_kind) -> Option<IcalProperty<'_>> {
        let property =
            unsafe { ical::icalcomponent_get_first_property(self.get_ptr(), property_kind) };
        if !property.is_null() {
            Some(IcalProperty::from_ptr(property, self.as_component()))
        } else {
            None
        }
    }

    fn get_properties(
        self: &Self,
        property_kind: ical::icalproperty_kind,
    ) -> Vec<IcalProperty<'_>> {
        let mut properties = Vec::new();
        unsafe {
            let mut property_ptr =
                ical::icalcomponent_get_first_property(self.get_ptr(), property_kind);
            while !property_ptr.is_null() {
                let property = IcalProperty::from_ptr(property_ptr, self.as_component());
                properties.push(property);
                property_ptr = ical::icalcomponent_get_next_property(self.get_ptr(), property_kind);
            }
        }
        properties
    }

    fn get_properties_all(&self) -> Vec<IcalProperty<'_>> {
        self.get_properties(ical::icalproperty_kind_ICAL_ANY_PROPERTY)
    }

    fn get_properties_by_name(&self, property_name: &str) -> Vec<IcalProperty> {
        let property_kind = unsafe {
            let c_str = CString::new(property_name).unwrap();
            ical::icalproperty_string_to_kind(c_str.as_ptr())
        };
        self.get_properties(property_kind)
    }

    fn get_property_by_name(&self, property_name: &str) -> Option<IcalProperty> {
        let property_kind = unsafe {
            let c_str = CString::new(property_name).unwrap();
            ical::icalproperty_string_to_kind(c_str.as_ptr())
        };
        self.get_property(property_kind)
    }

    unsafe fn remove_property_all(&self, kind: ical::icalproperty_kind) -> usize {
        unsafe fn remove_property_inner(
            comp: *mut ical::icalcomponent,
            kind: ical::icalproperty_kind,
        ) -> usize {
            let mut count = 0;
            let mut prop = ical::icalcomponent_get_first_property(comp, kind);
            while !prop.is_null() {
                ical::icalcomponent_remove_property(comp, prop);
                count += 1;
                prop = ical::icalcomponent_get_current_property(comp);
            }
            let mut inner_comp = ical::icalcomponent_get_first_component(
                comp,
                ical::icalcomponent_kind_ICAL_ANY_COMPONENT,
            );
            while !inner_comp.is_null() {
                count += remove_property_inner(inner_comp, kind);
                inner_comp = ical::icalcomponent_get_next_component(
                    comp,
                    ical::icalcomponent_kind_ICAL_ANY_COMPONENT,
                )
            }
            count
        }

        let comp = self.get_ptr();
        remove_property_inner(comp, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use crate::IcalVCalendar;

    #[test]
    fn get_property_test() {
        let cal = IcalVCalendar::from_str(testing::data::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();
        let prop_name = "SUMMARY";
        let prop_value: String = event.get_property_by_name(prop_name).unwrap().get_value();

        assert_eq!(
            "Festival International de Jazz de Montreal".to_string(),
            prop_value
        );
    }

    #[test]
    fn get_property_test_lastmodified() {
        let cal =
            IcalVCalendar::from_str(testing::data::TEST_EVENT_MULTIDAY_LASTMODIFIED, None).unwrap();
        let event = cal.get_principal_event();
        let prop_name = "LAST-MODIFIED";
        let prop_value: String = event.get_property_by_name(prop_name).unwrap().get_value();

        assert_eq!("20070423T123432Z".to_string(), prop_value);
    }

    #[test]
    fn get_property_test_cal() {
        let cal = IcalVCalendar::from_str(testing::data::TEST_EVENT_MULTIDAY, None).unwrap();
        let prop_name = "PRODID";
        let prop_value: String = cal.get_property_by_name(prop_name).unwrap().get_value();

        assert_eq!(
            "-//ABC Corporation//NONSGML My Product//EN".to_string(),
            prop_value
        );
    }

    #[test]
    fn get_property_test_negative() {
        let cal = IcalVCalendar::from_str(testing::data::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();
        let prop_name = "DESCRIPTION";
        let prop = event.get_property_by_name(prop_name);

        assert!(prop.is_none());
    }

    #[test]
    fn get_property_by_name_test() {
        let cal = IcalVCalendar::from_str(testing::data::TEST_EVENT_MULTIDAY, None).unwrap();
        let event = cal.get_principal_event();
        let prop_name = "NONSENSE";
        let prop = event.get_property_by_name(prop_name);

        assert!(prop.is_none());
    }
}
