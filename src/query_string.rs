use std::collections::BTreeMap;


/// Utility for parsing query strings
/// argument qs is a string of form key1=val1?key2=val2
pub fn parse_query_string(qs: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    if qs.len() > 0 {
        let key_value_pairs = qs.split('&');

        let mut equal_counter = 0;
        for kv_pair in key_value_pairs {
            let kv: Vec<&str> = kv_pair.split('=').collect();

            if kv.len() != 2 {
                continue;
            }

            let mut key = kv[0].to_string();
            let val = kv[1].to_string();

            if map.contains_key(&key) {
                key.push_str(format!("{}", equal_counter).as_str());
                equal_counter += 1;
            }
            map.insert(key, val);
        }
    }

    map
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_query() {
        
        let q1 = "";

        let result = parse_query_string(q1);

        assert_eq!(result.len(), 0);   
    }

    #[test]
    fn no_equal_sign() {
        
        let q1 = "hello";

        let result = parse_query_string(q1);

        assert_eq!(result.len(), 0);   
    }

    #[test]
    fn no_equal_sign_mixed() {
        
        let q1 = "hello&name=file";

        let result = parse_query_string(q1);

        assert_eq!(result.len(), 1);   

        assert_eq!(result["name"], "file");
    }
 
    #[test]
    fn one_query() {
        
        let q1 = "name=file.txt";

        let result = parse_query_string(q1);

        assert_eq!(result["name"], "file.txt");   
    }

    #[test]
    fn two_queries() {
        
        let q1 = "name=file.txt&container=here_be_files";

        let result = parse_query_string(q1);

        assert_eq!(result["name"], "file.txt");   
        assert_eq!(result["container"], "here_be_files");   
    }

    #[test]
    fn three_queries() {
        
        let q1 = "name=file.txt&container=here_be_files&options=read_only";

        let result = parse_query_string(q1);

        assert_eq!(result["name"], "file.txt");   
        assert_eq!(result["container"], "here_be_files");   
        assert_eq!(result["options"], "read_only");   
    }

}