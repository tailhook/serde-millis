use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};


use traits::{Sealed, Milliseconds};


impl Sealed for Duration {
    fn to_millis(&self) -> Option<u64> {
        self.as_secs().checked_mul(1000).and_then(|x| {
            x.checked_add((self.subsec_nanos()/1_000_000) as u64)
        })
    }
    fn from_millis(val: u64) -> Option<Self> where Self: Sized {
        Some(Duration::from_millis(val))
    }
}
impl Sealed for SystemTime {
    fn to_millis(&self) -> Option<u64> {
        self.duration_since(UNIX_EPOCH).ok().and_then(|x| x.to_millis())
    }
    fn from_millis(val: u64) -> Option<Self> where Self: Sized {
        Some(UNIX_EPOCH + Duration::from_millis(val))
    }
}
impl Sealed for Instant {
    fn to_millis(&self) -> Option<u64> {
        let inow = Instant::now();
        let snow = SystemTime::now();
        if *self < inow {
            (snow - inow.duration_since(*self)).to_millis()
        } else {
            (snow + self.duration_since(inow)).to_millis()
        }
    }
    fn from_millis(val: u64) -> Option<Self> where Self: Sized {
        let inow = Instant::now();
        let snow = SystemTime::now();
        let stime = UNIX_EPOCH + Duration::from_millis(val);
        if stime > snow {
            stime.duration_since(snow).ok().map(|x| inow + x)
        } else {
            snow.duration_since(stime).ok().map(|x| inow - x)
        }
    }
}

impl Milliseconds for Duration {}
impl Milliseconds for SystemTime {}
impl Milliseconds for Instant {}


#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
    use traits::Sealed;

    #[test]
    fn test_duration() {
        assert_eq!(Duration::new(1, 0).to_millis(), Some(1000));
        assert_eq!(Duration::new(1234, 0).to_millis(), Some(1234000));
        assert_eq!(Duration::new(1, 2300000).to_millis(), Some(1002));
        assert_eq!(<Duration as Sealed>::from_millis(1002),
            Some(Duration::new(1, 2000000)));
        assert_eq!(<Duration as Sealed>::from_millis(1000),
            Some(Duration::new(1, 0)));
        assert_eq!(<Duration as Sealed>::from_millis(1234000),
            Some(Duration::new(1234, 0)));
    }

    #[test]
    fn test_systemtime_past() {
        assert_eq!(SystemTime::from_millis(
            (UNIX_EPOCH + Duration::from_millis(1511885454870))
            .to_millis().unwrap()).unwrap(),
            UNIX_EPOCH + Duration::from_millis(1511885454870))
    }

    #[test]
    fn test_systemtime_future() {
        assert_eq!(SystemTime::from_millis(
            (UNIX_EPOCH + Duration::from_millis(2147483647000))
            .to_millis().unwrap()).unwrap(),
            UNIX_EPOCH + Duration::from_millis(2147483647000))
    }

    #[test]
    fn test_instant_past() {
        let early = Instant::now();
        let converted = Instant::from_millis(
            (Instant::now() - Duration::from_millis(2000))
            .to_millis().unwrap()).unwrap();
        let late = Instant::now();
        println!("Conv {:?} {:?} {:?}", converted, early, late);
        assert!(converted >= early - Duration::from_millis(2002));
        assert!(converted <= late - Duration::from_millis(1998));
    }

    #[test]
    fn test_instant_future() {
        let early = Instant::now();
        let converted = Instant::from_millis(
            (Instant::now() + Duration::from_millis(2000))
            .to_millis().unwrap()).unwrap();
        let late = Instant::now();
        println!("Conv {:?} {:?} {:?}", converted, early, late);
        assert!(converted >= early + Duration::from_millis(1998));
        assert!(converted <= late + Duration::from_millis(2002));
    }

}
