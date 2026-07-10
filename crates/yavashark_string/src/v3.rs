
// TODO: string v3 should store the string data inline to the definition
//       this will allow for faster access to the string data and less memory allocations
// There will be one pointer and which has an tag to what the string actually is. That can be
// - InlineAscii
// - InlineWtf16
// - Slice
// - Rope
// - External
// The data will be unsized, since all strings will be now stored inline.