use http::{HeaderMap, header::COOKIE};

pub fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {
    let mut jar = cookie::CookieJar::new();
    let Some(cookie) = headers.get(COOKIE) else {
        return jar;
    };
    let Ok(cookie_str) = cookie.to_str() else {
        return jar;
    };

    for cookie in cookie::Cookie::split_parse_encoded(cookie_str) {
        let Ok(cookie) = cookie else { continue };
        jar.add_original(cookie.into_owned());
    }

    jar
}

pub fn is_admin(headers: &HeaderMap) -> bool {
    // extra.insert(COOKIE, "hello".parse().unwrap());
    let Some(cookie) = headers.get(COOKIE) else {
        return false;
    };
    // cookie.to_str()
    todo!()
}
// export function getUser(
//   cookies: AstroCookies
// ): (CookieUser & { isAdmin: boolean }) | null {
//   const userInfoCookie = cookies.get(USER_INFO_COOKIE);
//   if (!userInfoCookie) {
//     return null;
//   }

//   try {
//     const user = JSON.parse(userInfoCookie.value);
//     return {
//       ...user,
//       isAdmin: isAdmin(user.id),
//     };
//   } catch {
//     return null;
//   }
// }
