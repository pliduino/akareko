macro_rules! icon {
    ($name:ident, $path:literal) => {
        pub const $name: &[u8] = include_bytes!($path);
    };
}

icon!(EYE_SLASH_ICON, "../../assets/icons/eye-slash.svg");
icon!(EYE_ICON, "../../assets/icons/eye.svg");
icon!(CHAT_ICON, "../../assets/icons/chat.svg");
icon!(DOWNLOAD_ICON, "../../assets/icons/download-simple.svg");
icon!(CHECK_CIRCLE_ICON, "../../assets/icons/check-circle.svg");
icon!(BOOK_BOOKMARK_ICON, "../../assets/icons/book-bookmark.svg");

icon!(BOOKMARK_SIMPLE, "../../assets/icons/bookmark-simple.svg");
icon!(
    BOOKMARK_SIMPLE_FILL,
    "../../assets/icons/bookmark-simple-fill.svg"
);

icon!(
    DOTS_THREE_VERTICAL_ICON,
    "../../assets/icons/dots-three-vertical.svg"
);
icon!(ARROW_LEFT_ICON, "../../assets/icons/arrow-left.svg");
icon!(ARROW_RIGHT_ICON, "../../assets/icons/arrow-right.svg");
icon!(PLUS_ICON, "../../assets/icons/plus.svg");
icon!(CIRCLE, "../../assets/icons/circle-fill.svg");
