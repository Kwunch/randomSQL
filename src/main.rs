#![allow(non_snake_case)]
#![allow(unused_assignments)]

use chrono::{NaiveDate, NaiveDateTime};
use fake::{
    faker::{address::en::*, company::en::*, internet::en::*, name::raw::*, phone_number::en::*},
    locales::*,
    Fake,
    Faker,
    // More modules for mock data found at
    // https://docs.rs/fake/latest/fake/faker/index.html
};

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io,
    io::{stdout, Write},
};

fn check_compound_attribute(index: usize, attribute_checker: &Vec<String>) -> bool {
    /*
        * Checks if the compound attribute is valid
        * Compound attributes are of the form: (attribute1, attribute2, attribute3, ...)
        * Each attribute is separated by a semicolon

        :Parameters:
            - `index`: The index of the compound attribute in attribute_checker
            - `attribute_checker`: The vector of attributes to check

        :Returns:
            - `bool`: True if valid, false if invalid
    */

    // First, remove the first and last parenthesis of attribute_checker[2]
    let compound_attribute =
        attribute_checker[index][1..attribute_checker[index].len() - 1].to_string();

    // Split compound_attribute into a vector of attributes. Delimiter: ';'
    let compound_attributes = compound_attribute.split(";").collect::<Vec<&str>>();

    //New compound_attributes Vec -> [FIRST_NAME VARCHAR(20), MIDDLE_INITIAL CHAR(1), LAST_NAME VARCHAR(20)]
    //Iterate through compound_attributes, trim each element and check if each attribute is valid
    for compound_attribute in compound_attributes {
        let compound_attribute = compound_attribute.trim();
        let compound_attribute = compound_attribute.split(" ").collect::<Vec<&str>>();
        if compound_attribute.len() != 2 {
            println!("[!] Invalid Compound Attribute");
            return false;
        }
        if !check_data_type(compound_attribute[1].to_uppercase().as_str()) {
            println!("[!] Invalid Data Type");
            return false;
        }
    }
    true
}

fn check_data_type(attribute_type: &str) -> bool {
    /*
       * Checks if the data type is valid
       * Returns true if valid, false if invalid
       * Custom data types made for this program: EMAIL, GROUP, PHONE, SSN, STATE, ZIP, STREET-ADDRESS, FULL-ADDRESS, NAME, PASSWORD, USERNAME

       :Parameters:
           - `attribute_type`: The data type to check
       :Returns:
           - `bool`: True if valid, false if invalid

    */

    //Recreate valid types variable, but put all types in alphabetical order
    let valid_types = [
        "BIGINT",
        "BIT",
        "BOOLEAN",
        "BOX",
        "BYTEA",
        "CHAR",
        "CIDR",
        "CIRCLE",
        "CITY_SHORT",
        "CITY_US",
        "COMPANYNAME",
        "COMPOUND", // Not a real data type, used for compound attributes
        "COUNTRY",
        "DATE",
        "DECIMAL",
        "DOUBLE PRECISION",
        "EMAIL",
        "ENUM",
        "FLOAT4",
        "FLOAT8",
        "GROUP",
        "INDUSTRY",
        "INET",
        "INTEGER",
        "INTERVAL",
        "JSON",
        "JSONB",
        "LINE",
        "LSEG",
        "MACADDR",
        "MONEY",
        "NAME",
        "NUMERIC",
        "PASSWORD",
        "PATH",
        "PG_LSN",
        "PHONE",
        "POINT",
        "POLYGON",
        "PROFESSION",
        "REAL",
        "SERIAL",
        "SMALLINT",
        "SSN",
        "STATE",
        "STATE_US",
        "STREET_ADDRESS",
        "STREET_NAME_US",
        "TEXT",
        "TIME",
        "TIMESTAMP",
        "TSQUERY",
        "TSVECTOR",
        "TXID_SNAPSHOT",
        "USERNAME",
        "UUID",
        "VARCHAR",
        "XML",
        "ZIP_US",
    ];

    // Make sure the data type is valid
    valid_types
        .iter()
        .any(|&substring| attribute_type.starts_with(substring))
}

fn check_key_definition(key_def: &str) -> bool {
    /*
        * Checks if the key definition is properly defined
        * PK -> Primary Key
        * FK -> Foreign Key
        * AK -> Alternate (Unique) Key
        * PK/FK -> Primary Key and Foreign Key
        * AK/FK -> Alternate (Unique) Key and Foreign Key

        :parameters:
            - `key_def`: The key definition to check
        :returns:
            - `bool`: True if valid, false if invalid
    */

    let valid_keys = ["PK", "FK", "AK", "PK/FK", "AK/FK"];

    valid_keys.contains(&key_def)
}

fn check_pair(
    generated_pair_vector: &Vec<String>,
    previous_pairs: &Vec<Vec<String>>,
    table_attributes: &Vec<String>,
    uq_attributes: &HashMap<String, Vec<String>>,
    count: usize,
) -> (bool, Vec<String>) {
    /*
        * Recursively calls itself until it generates a valid composite key pair
        * All data used in pair generation comes from uq_attributes

        :parameters:
            - `generated_pair_vector`: The vector of generated pairs
            - `previous_pairs`: The vector of previously generated pairs
            - `table_attributes`: The vector of table attributes
            - `uq_attributes`: The hashmap of unique attributes
            - `count`: The number of attributes in the composite key

        :returns:
            - `bool`: True if valid, false if invalid
            - `Vec<String>`: The generated pair
    */
    let mut new_pair: Vec<String> = Vec::new();

    // First Time Running With Generated Pair
    let pair_changed = count > 0;

    let reverse_generated_pair: Vec<String> = generated_pair_vector
        .clone()
        .into_iter()
        .rev()
        .collect::<Vec<String>>();

    if previous_pairs.contains(&generated_pair_vector)
        || previous_pairs.contains(&reverse_generated_pair)
    {
        // Duplicate Pair Found
        // Must Generate New Data For Generated Pair
        for temp_attr in table_attributes.clone() {
            let temp_attr_list = temp_attr.trim().split(' ').collect::<Vec<&str>>();
            if temp_attr_list[0].to_uppercase() == "PK/FK" {
                let temp_attribute_name = temp_attr_list[3]
                    .split('(')
                    .nth(1)
                    .unwrap()
                    .replace(")", "");

                if let Some(history) = uq_attributes.get(&temp_attribute_name) {
                    loop {
                        if let Some(value) = history.choose(&mut thread_rng()) {
                            if !new_pair.contains(&value.to_string()) {
                                new_pair.push(value.to_string()); // Cast to &str
                                break;
                            }
                        }
                    }
                }
            }
        }
        return check_pair(
            &new_pair,
            previous_pairs,
            table_attributes,
            uq_attributes,
            count + 1,
        );
    }

    new_pair.extend_from_slice(&generated_pair_vector);
    (pair_changed, new_pair)
}


fn create_insert_statement(
    table_name: &str,
    table_attributes: &Vec<String>,
    statement_data: &HashMap<String, String>,
) -> String {
    /*
        * Creates the insert statement for the table
        * Example: INSERT INTO table_name VALUES (data1, data2, data3, ...)
        * The data is generated from the statement_data hashmap

        :parameters:
            - `table_name`: The name of the table
            - `table_attributes`: The vector of table attributes
            - `statement_data`: The hashmap of generated data for the table

        :returns:
            - `String`: The insert statement for the table

        TODO - Optimize this disgusting code
    */
    let mut insert_string = format!("INSERT INTO {} VALUES (", table_name);

    // Iterate over the generated data and add it to the INSERT statement
    // useless_key has no purpose and is only there to aid in iteration.
    for (index, attribute) in table_attributes.iter().enumerate() {
        let mut isCompound: bool = false;
        let compound_attribute;
        let mut compound_attribute_string = format!("(");
        let mut attribute_name: Vec<&str> = attribute.trim().split(" ").collect();

        if attribute_name.contains(&"COMPOUND") || attribute_name.contains(&"compound") {
            isCompound = true;

            // Join all elements after the element containing "COMPOUND" into one string element
            // "COMPOUND" can either be at attribute_checker[1] or attribute_checker[2]
            // Attribute checker should then have either 3 elements or 4 elements
            // 3 elements -> [attribute_name, COMPOUND, compound_attributes]
            // 4 elements -> [key_definition, attribute_name, COMPOUND, compound_attributes]
            let index_of_compound = attribute_name
                .iter()
                .position(|&x| x == "compound" || x == "COMPOUND")
                .unwrap(); // Get index of "compound"

            compound_attribute = attribute_name[index_of_compound + 1..].join(" "); // Merge compound attr into one string
            attribute_name.truncate(index_of_compound + 1); // Remove excess elements
            attribute_name.push(&compound_attribute); // Push new compound attribute to end
        }

        let mut data: String = "".to_string();

        match attribute_name.len() {
            1 | 2 => {
                data = statement_data
                    .get(attribute_name[0])
                    .unwrap()
                    .trim()
                    .to_string();
            }
            3 => match isCompound {
                true => {
                    data = statement_data
                        .get(attribute_name[0])
                        .unwrap()
                        .trim()
                        .to_string();
                }
                false => {
                    data = statement_data
                        .get(attribute_name[1])
                        .unwrap()
                        .trim()
                        .to_string();
                }
            }
            4 | 5 => {
                /*
                    * Examples of Attribute
                    * PK/FK userID INTEGER profile(userID)
                    * [PK/AK] full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20))
                    * [FK] MBR COMPOUND (x_min INTEGER, x_max INTEGER, y_min INTEGER, y_max INTEGER) region(coordinates)
                */
                data = statement_data
                    .get(attribute_name[1])
                    .unwrap()
                    .trim()
                    .to_string();
            }
            _ => {}
        }

        match isCompound {
            true => {
                let compound_attributes = data.split(", ").collect::<Vec<&str>>();
                for (cIndex, cAttribute) in compound_attributes.iter().enumerate() {
                    if cIndex == compound_attributes.len() - 1 {
                        if cAttribute.parse::<f64>().is_ok() {
                            compound_attribute_string += &format!("{})", cAttribute);
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            compound_attribute_string += "NULL)";
                        } else if cAttribute == &"0" {
                            compound_attribute_string += "0)";
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            compound_attribute_string += "TRUE)";
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            compound_attribute_string += "FALSE)";
                        } else {
                            compound_attribute_string += &format!("\'{}\')", cAttribute);
                        }
                    } else {
                        if cAttribute.parse::<f64>().is_ok() {
                            compound_attribute_string += &format!("{},", cAttribute);
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            compound_attribute_string += "NULL,";
                        } else if cAttribute == &"0" {
                            compound_attribute_string += "0,";
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            compound_attribute_string += "TRUE,";
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            compound_attribute_string += "FALSE,";
                        } else {
                            compound_attribute_string += &format!("\'{}\',", cAttribute);
                        }
                    }
                }
                if index == table_attributes.len() - 1 {
                    insert_string += &format!("{});", compound_attribute_string);
                } else {
                    insert_string += &format!("{}, ", compound_attribute_string);
                }
            }
            false => {
                // If we're at the last piece of data, close the statement with a semicolon
                if index == table_attributes.len() - 1 {
                    if data.parse::<f64>().is_ok() {
                        insert_string += &format!("{});", data);
                    } else if data.to_ascii_uppercase() == "NULL" {
                        insert_string += "NULL);";
                    } else if data == "0" {
                        insert_string += "0);";
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        insert_string += "TRUE);";
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        insert_string += "FALSE);";
                    } else {
                        insert_string += &format!("\'{}\');", data);
                    }
                    // If we're not at the last piece of data, add a comma to the end of the statement
                } else {
                    if data.parse::<f64>().is_ok() {
                        insert_string += &format!("{}, ", data);
                    } else if data.to_ascii_uppercase() == "NULL" {
                        insert_string += "NULL, ";
                    } else if data == "0" {
                        insert_string += "0, ";
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        insert_string += "TRUE, ";
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        insert_string += "FALSE, ";
                    } else {
                        insert_string += &format!("\'{}\', ", data);
                    }
                }
            }
        }
    }
    insert_string
}

fn get_random_data(
    attribute_type: &str,
    optional_data_size: Option<Vec<u16>>,
    statement_data: &HashMap<String, String>,
) -> String {
    /*
        * Generates random data for the attribute type
        * If the attribute type is a custom data type, the custom data type is generated
        * If the attribute type is a default data type, the default data type is generated

        ** Uses the Faker library to generate data for default data types

        :parameters:
            - `attribute_type`: The type of the attribute
            - `optional_data_size`: The optional data size for the attribute
            - `statement_data`: The hashmap of generated data for the table

        :returns:
            - `String`: The generated data for the attribute

        :TODO:
            - Add First Name, Last Name, Middle Name / Middle Initial
    */
    match attribute_type {
        char_type if char_type.starts_with("CHAR") || char_type.starts_with("VARCHAR") => {
            //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
            let char_size = optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(3..12)])
                .get(0)
                .unwrap()
                .to_owned();
            Faker
                .fake::<String>()
                .chars()
                .take(char_size as usize)
                .collect::<String>()
        }
        decimal_type if decimal_type.starts_with("DECIMAL") => {
            //Attempt to unwrap and get vec[0] and vec[1] of optional data-size
            //Vec[0] is the number of digits before the decimal point
            //Vec[1] is the number of digits after the decimal point
            //If none, set random values of each
            let unwrapped_decimal = optional_data_size.unwrap_or_else(|| {
                vec![thread_rng().gen_range(3..12), thread_rng().gen_range(3..12)]
            });
            let digits_before_decimal = unwrapped_decimal.get(0).unwrap().to_owned();
            let digits_after_decimal = unwrapped_decimal.get(1).unwrap().to_owned();
            //Create decimal value from digits_before_decimal and digits_after_decimal
            let decimal_value = format!(
                "{}.{}",
                thread_rng().gen_range(0..10_u64.pow(digits_before_decimal as u32)),
                thread_rng().gen_range(0..10_u64.pow(digits_after_decimal as u32)),
            );
            decimal_value
        }
        money_type if money_type.starts_with("MONEY") => {
            //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
            let dollar_size = optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(3..12)])
                .get(0)
                .unwrap()
                .to_owned();

            // Generate a dollar amount between 3 figures and either dollar size or 12 figures
            let dollar_amount = thread_rng().gen_range(0..10i32.pow(dollar_size as u32));
            let cents_amount = thread_rng().gen_range(0..100);
            // Create decimal value from dollar_amount and cents_amount
            let decimal_value = format!("{}.{}", dollar_amount, cents_amount);
            decimal_value
        }
        name_type if name_type.starts_with("NAME") => {
            let name = Name(EN).fake::<String>();
            name.replace("'", "")
        }
        password_type if password_type.starts_with("PASSWORD") => Password(std::ops::Range {
            start: 8,
            end: (optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(8..12)])
                .get(0)
                .unwrap()
                .to_owned() as usize), //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
        })
            .fake(),
        username_type if username_type.starts_with("USERNAME") => {
            let first_name = FirstName(EN).fake::<String>();
            let last_name = LastName(EN).fake::<String>();
            let mut username = format!("{}{}", first_name, last_name);
            //Check if optional data size is specified in vec[0], if so, truncate username to that size if it is larger
            if let Some(size) = optional_data_size {
                if username.len() > size[0] as usize {
                    username.truncate(size[0] as usize);
                }
            }

            username.replace("'", "")
        }
        "INTEGER" => Faker.fake::<u16>().to_string(),
        "BOOLEAN" => Faker.fake::<bool>().to_string(),
        "DATE" => {
            let year = thread_rng().gen_range(1900..2021);
            let month = thread_rng().gen_range(1..13);
            let day = thread_rng().gen_range(1..29);
            NaiveDate::from_ymd_opt(year, month, day)
                .unwrap()
                .to_string()
        }
        "TIMESTAMP" => {
            let year = thread_rng().gen_range(1900..2021);
            let month = thread_rng().gen_range(1..13);
            let day = thread_rng().gen_range(1..29);
            let hour = thread_rng().gen_range(0..24);
            let minute = thread_rng().gen_range(0..60);
            let second = thread_rng().gen_range(0..60);
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap(),
            )
                .to_string()
        }
        "TIME" => {
            let hour = thread_rng().gen_range(0..24);
            let minute = thread_rng().gen_range(0..60);
            let second = thread_rng().gen_range(0..60);
            chrono::NaiveTime::from_hms_opt(hour, minute, second)
                .unwrap()
                .to_string()
        }
        "GROUP" => ["Member", "Mod"]
            .choose(&mut thread_rng())
            .unwrap()
            .to_string(),
        "EMAIL" => {
            let domains = vec![
                "@outlook.com",
                "@gmail.com",
                "@pitt.edu",
                "@yahoo.com",
                "@proton.mail",
                "@pm.me",
                "@paranoid.email",
            ];
            let username = match (
                statement_data.get(&"name".to_string()),
                statement_data.get(&"full name".to_string()),
                statement_data.get(&"full_name".to_string()),
            ) {
                (Some(name), _, _) => name.replace(" ", ""),
                (_, Some(full_name), _) => full_name.replace(" ", ""),
                (_, _, Some(full_name_underscore)) => full_name_underscore.replace(" ", ""),
                _ => {
                    let name = Name(EN).fake::<String>();
                    name.replace("'", "").to_string();
                    name.replace(" ", "").to_string()
                }
            };
            format!("{}{}", username, domains.choose(&mut thread_rng()).unwrap())
        }
        "STATE_US" => {
            //Generate random state in US using faker
            let state = StateName().fake::<String>();
            state.replace("'", "")
        }
        "CITY_US" => {
            let city = CityName().fake::<String>();
            city.replace("'", "")
        }
        "CITY_SHORT" => {
            let city_prefix = CityPrefix().fake::<String>();
            city_prefix.replace("'", "")
        }
        "STREET_NAME_US" => {
            let street_address = StreetName().fake::<String>();
            street_address.replace("'", "")
        }
        "ZIP_US" => {
            let zip = ZipCode().fake::<String>();
            zip.replace("'", "")
        }
        "SSN" => {
            // Generate random 9 digit number
            let ssn = thread_rng().gen_range(100_000_000..1_000_000_000);
            ssn.to_string()
        }
        "PHONE" => {
            // Generate random phone number using faker
            let phone_number = PhoneNumber().fake::<String>();
            phone_number.replace("'", "")
        }
        "COUNTRY" => {
            // Generate random country using faker
            let country = CountryName().fake::<String>();
            country.replace("'", "")
        }
        "COMPANYNAME" => {
            // Generate random company name using faker
            let company_name = CompanyName().fake::<String>();
            company_name.replace("'", "")
        }
        "INDUSTRY" => {
            // Generate random industry using faker
            let industry = Industry().fake::<String>();
            industry.replace("'", "")
        }
        "PROFESSION" => {
            // Generate random profession using faker
            let profession = Profession().fake::<String>();
            profession.replace("'", "")
        }
        _ => {
            panic!("Unknown Type In Data Generation! {}", attribute_type);
        }
    }
}

fn get_references(attribute_definition: &Vec<String>, index: usize) -> (String, String) {
    /*
        * Gets the referenced table and attribute from the attribute definition
        * Example: PK userID INTEGER profile(userID)
        * Referenced Table: profile
        * Referenced Attribute: userID

        :parameters:
            - `attribute_definition`: The vector of attribute definitions
            - `index`: The index of the attribute definition to get the referenced table and attribute from

        :returns:
            - `String`: The referenced table
            - `String`: The referenced attribute
    */
    let referenced_table = attribute_definition[index]
        .split('(')
        .next()
        .unwrap()
        .to_string();

    let referenced_attribute = attribute_definition[index]
        .split('(')
        .nth(1)
        .unwrap()
        .replace(")", "");

    (referenced_table, referenced_attribute)
}

fn get_referenced_attribute(
    attribute_list: &Vec<String>,
    referenced_attribute: &str,
) -> Option<String> {
    /*
        *Returns the table attribute that is referenced by the foreign key
        *Returns None if the referenced attribute is not found

        :parameters:
            - `attribute_list`: The list of attributes in the table
            - `referenced_attribute`: The referenced attribute to find

        :returns:
            - `Option<String>`: The referenced attribute or None
    */

    if !attribute_list.is_empty() {
        for attribute in attribute_list {
            if attribute.split_whitespace().nth(1) == Some(referenced_attribute) {
                return Some(attribute.to_string());
            }
        }
    }
    None
}

fn generate_mock_data(
    tables: &Vec<String>,
    key_dictionary: &HashMap<String, Vec<String>>,
    reference_dictionary: &HashMap<String, Vec<HashMap<String, String>>>,
    path: &String,
    iterations: u16,
) {
    /*
        * Generates the mock data for the tables
        * Writes the mock data to the file specified by path

        :parameters:
            - `tables`: The vector of tables to generate mock data for
            - `key_dictionary`: The hashmap of keys for each table
            - `reference_dictionary`: The hashmap of references for each table
            - `path`: The path to write the mock data to
            - `iterations`: The number of iterations to generate mock data for
    */

    let mut unique_attribute_checker: HashMap<String, Vec<String>> = HashMap::new();
    let mut unique_pair_checker: HashMap<String, Vec<Vec<String>>> = HashMap::new();

    let mut statements_generated: u16 = 0;

    print!(
        "Generating SQL Inserts: {}/{} Created",
        statements_generated, iterations
    );
    stdout().flush().unwrap();

    for table in tables.into_iter() {
        let mut pairwise_table = false;

        let table: Vec<&str> = table.split_whitespace().collect();

        let num_statements = table[0].parse::<i32>().unwrap(); // Keeps Track of Number of Statements to Generate
        let table_name = String::from(table[1]); //Table name
        let mut table_attributes: String = table[2..] //Table attributes
            .join(" ")
            .to_string();

        table_attributes.remove(0); // Remove beginning parenthesis
        table_attributes.remove(table_attributes.len() - 1); // Remove ending parenthesis

        let table_attributes: Vec<String> = table_attributes // Split attributes -> example element: 'PK userID INTEGER'
            .clone()
            .split(",")
            .map(|s| s.to_owned())
            .collect();

        let check_references = reference_dictionary.get(&table_name);
        let attribute_order_keeper: Vec<String> = table_attributes.clone(); // Keeps Track of Order of Attributes. Used for Creating Insert Statements

        let mut primary_keys: Vec<String> = Vec::new();

        // Check if table is a composite keyed table
        if let Some(_check_references) = check_references {
            for attribute in &table_attributes {
                if attribute.trim()[..2].to_uppercase() == "PK".to_string() {
                    primary_keys.push(attribute.to_string());
                }
            }
            if primary_keys.len() > 1 {
                pairwise_table = true;
            }
        }

        for _ in 0..num_statements {
            // Stores generated value for CURRENT Insert statement. Resets after each insert statement is generated
            let mut statement_data: HashMap<String, String> = HashMap::new();

            // Keep track of previous attribute [Used for tables that reference the same table attribute multiple times]
            // (PK/FK userID1 INTEGER profile(userID), PK/FK userID2 INTEGER profile(userID)
            let mut referenced_attributes: HashMap<String, Vec<String>> = HashMap::new();

            // Used in pairwise (composite) key scenarios
            let mut pair_list: Vec<String> = Vec::new();

            for attribute in &table_attributes {
                let pairwise_attribute =
                    if pairwise_table && primary_keys.contains(&attribute.to_string()) {
                        true
                    } else {
                        false
                    };

                let mut attribute_definition: Vec<String> = attribute
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect();

                let isCompound = merge_compound(&mut attribute_definition);

                let attribute_definition = attribute_definition;

                match &attribute_definition.len() {
                    1 => {
                        /*
                         * User requests default value for attribute
                         * Does not need to specify attribute
                         * For instance if table has attribute that would be NULL for mock data user can put "NULL"
                         * Then for each insert, that attribute will be NULL for
                         */
                        match attribute_definition[0].to_uppercase().as_str() {
                            // TODO - Add more default values
                            "0" | "NULL" | "TRUE" | "FALSE" => {
                                statement_data.insert(
                                    attribute_definition[0].to_string(),
                                    attribute_definition[0].to_string(),
                                );
                            }
                            _ => {
                                println!("[!] Invalid default value for attribute");
                                continue;
                            }
                        }
                    }
                    2 => {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [attribute name] [attribute type]
                            * Example: 'userID INTEGER'

                            * Generate Data For Attribute
                        */

                        let attribute_name = attribute_definition[0].to_string();
                        let attribute_type = attribute_definition[1].to_uppercase().to_string();

                        let optional_variable_size: Option<Vec<u16>> =
                            set_variable_size(&attribute_type);

                        let generated_data: String = get_random_data(
                            &attribute_type,
                            optional_variable_size.clone(),
                            &statement_data,
                        );

                        statement_data.insert(attribute_name, generated_data);
                    }
                    3 => match isCompound {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [key definition] [attribute name] [attribute type]
                            * OR
                            * [attribute name] [compound] [(compound attribute)]
                            * Standard Example: 'PK userID INTEGER'
                            * Compound Example: 'full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20)

                            * Generate Data For Attribute
                        */
                        true => {
                            /*
                             * Compound Attribute. Generate Data For Each Attribute In The Compound Attribute
                             * Example: 'full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20))'
                             */
                            let attribute_name = attribute_definition[0].to_string();
                            // Remove beginning and ending parenthesis of compound attribute
                            let compound_attribute = attribute_definition[2]
                                [1..attribute_definition[2].len() - 1]
                                .to_string();
                            let compound_attribute =
                                compound_attribute.split("; ").collect::<Vec<&str>>();

                            let mut compound_attribute_data: Vec<String> = Vec::new();

                            // Iterate over each attribute in the compound attribute, get type and size, then generate data
                            for attribute in compound_attribute {
                                let attribute =
                                    attribute.split_whitespace().collect::<Vec<&str>>();
                                let attribute_type =
                                    attribute[1].trim().to_uppercase().to_string();

                                let optional_variable_size: Option<Vec<u16>> =
                                    set_variable_size(&attribute_type);

                                let generated_data: String = get_random_data(
                                    &attribute_type,
                                    optional_variable_size.clone(),
                                    &statement_data,
                                );

                                compound_attribute_data.push(generated_data);
                            }

                            // Create string of compound attribute data in form (data1, data2, data3, ...)
                            let compound_attribute_data = compound_attribute_data.join(", ");
                            statement_data.insert(attribute_name, compound_attribute_data);
                        }
                        false => {
                            /*
                             * Standard Attribute. Proceed As Before
                             */
                            let attribute_name = attribute_definition[1].to_string();
                            let attribute_type =
                                attribute_definition[2].to_uppercase().to_string();

                            let optional_variable_size: Option<Vec<u16>> =
                                set_variable_size(&attribute_type);

                            let mut generated_data: String = get_random_data(
                                &attribute_type,
                                optional_variable_size.clone(),
                                &statement_data,
                            );

                            if unique_attribute_checker.contains_key(&attribute_name) {
                                while (unique_attribute_checker[&attribute_name])
                                    .contains(&generated_data)
                                {
                                    generated_data = get_random_data(
                                        &attribute_type,
                                        optional_variable_size.clone(),
                                        &statement_data,
                                    );
                                }
                            }

                            unique_attribute_checker
                                .entry(attribute_name.clone())
                                .or_insert(Vec::new())
                                .push(generated_data.clone());

                            statement_data.insert(attribute_name, generated_data.clone());

                            if pairwise_attribute {
                                pair_list.push(generated_data);
                            }
                        }
                    }
                    4 => match isCompound {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [foreign key definition] [attribute name] [attribute type] [foreign table]
                            * OR
                            * [primary/unique key] [attribute name] [compound] [(compound attribute)]
                            * Example: 'PK/FK userID INTEGER profile(userID)'

                            * Generate Data For Attribute
                        */
                        true => {
                            /*
                                * Keyed Compound Attribute.
                                * Generate Data For Each Attribute In The Compound Attribute
                                * Add to unique_attribute_checker
                            */
                            let comp_attr_name = attribute_definition[1].to_string();
                            let comp_attr_compound = attribute_definition[3].to_string();

                            let comp_attr_compound =
                                comp_attr_compound[1..comp_attr_compound.len() - 1].to_string();

                            let comp_attr_compound =
                                comp_attr_compound.split("; ").collect::<Vec<&str>>();

                            loop {
                                let mut compound_attribute_data: Vec<String> = Vec::new();
                                // Iterate over each attribute in the compound attribute, get type and size, then generate data
                                for attribute in comp_attr_compound.iter() {
                                    let attribute =
                                        attribute.split_whitespace().collect::<Vec<&str>>();
                                    let attribute_type =
                                        attribute[1].trim().to_uppercase().to_string();

                                    let optional_variable_size: Option<Vec<u16>> =
                                        set_variable_size(&attribute_type);

                                    let generated_data: String = get_random_data(
                                        &attribute_type,
                                        optional_variable_size.clone(),
                                        &statement_data,
                                    );

                                    compound_attribute_data.push(generated_data);
                                }

                                // Create string of compound attribute data in form (data1, data2, data3, ...)
                                let compound_attribute_data =
                                    compound_attribute_data.join(", ");

                                // If primary or unique key, add to unique_attribute_checker
                                if unique_attribute_checker.contains_key(&comp_attr_name) &&
                                    unique_attribute_checker[&comp_attr_name].contains(&compound_attribute_data) {
                                    continue;
                                }

                                // Add to unique_attribute_checker
                                unique_attribute_checker
                                    .entry(comp_attr_name.clone())
                                    .or_insert(Vec::new())
                                    .push(compound_attribute_data.clone());

                                // Add to statement_data
                                statement_data.insert(
                                    comp_attr_name.clone(),
                                    compound_attribute_data.clone(),
                                );

                                // If pairwise, add to pair_list
                                if pairwise_attribute {
                                    if pair_list.contains(&compound_attribute_data) {
                                        continue;
                                    }
                                    pair_list.push(compound_attribute_data.clone());
                                }
                                break;
                            }
                        }
                        false => {
                            /*
                                * Standard Foreign Key Attribute.
                                * Generate Data For Attribute Based On Referenced Attribute
                                * I.E. Use Previously Generated Data From Referenced Attribute
                            */
                            let (referenced_table, referenced_attribute) =
                                get_references(&attribute_definition, 3);

                            if get_referenced_attribute(
                                key_dictionary.get(&referenced_table).unwrap(),
                                &referenced_attribute.to_uppercase().to_string(),
                            )
                                .is_none()
                            {
                                println!(
                                    "\nPROGRAM ERROR IN GENERATING DATA [Getting Referenced Attribute]"
                                );
                                std::process::exit(1);
                            }

                            loop {
                                /*
                                    * Reference does exist and is valid.
                                    * Get a random reference for that attribute from the list
                                 */
                                let randomized_data = unique_attribute_checker
                                    .get(&referenced_attribute.to_string())
                                    .unwrap()
                                    .choose(&mut thread_rng())
                                    .unwrap()
                                    .to_string();

                                /*
                                    * Check if the randomized data is the same as the previous data
                                    * If it is, generate new data
                                    * [key definition][0] [attribute name][1] [attribute type][2] [foreign table][3]
                                 */
                                if referenced_attributes
                                    .contains_key(&referenced_attribute.to_string())
                                {
                                    if referenced_attributes
                                        .get(&attribute_definition[3].to_string())
                                        .unwrap()
                                        .contains(&attribute_definition[1].to_string())
                                    {
                                        println!("Data being generated for same attribute.");
                                        println!("Program error.");
                                        println!("Exiting program.");

                                        std::process::exit(1);
                                    } else {
                                        if referenced_attributes
                                            .get(&attribute_definition[3].to_string())
                                            .unwrap()
                                            .contains(&randomized_data)
                                        {
                                            continue;
                                        }
                                    }
                                }

                                /*
                                    * True if table is composite keyed
                                    * Pair list can't have same data for keyed attributes if it does, regenerate data for current attribute
                                */
                                if pairwise_attribute {
                                    if pair_list.contains(&randomized_data) {
                                        continue;
                                    }
                                    pair_list.push(randomized_data.clone());
                                } else {
                                    //Not pairwise, check if data is unique for attribute or continue loop and generate new data for attribute
                                    if attribute_definition[0].starts_with("PK")
                                        || attribute_definition[0].starts_with("AK")
                                    {
                                        if unique_attribute_checker
                                            .contains_key(&attribute_definition[1])
                                            && unique_attribute_checker
                                            [&attribute_definition[1]]
                                            .contains(&randomized_data)
                                        {
                                            continue;
                                        }

                                        unique_attribute_checker
                                            .entry(attribute_definition[1].to_string())
                                            .or_insert(Vec::new())
                                            .push(randomized_data.clone());
                                    }
                                }

                                /*
                                    * Add the attribute to the list of referenced attributes
                                    * Add the data to the statement data
                                    * Break out of the loop
                                 */
                                referenced_attributes
                                    .entry(attribute_definition[3].to_string())
                                    .or_insert(Vec::new())
                                    .push(attribute_definition[1].to_string());

                                statement_data.insert(
                                    attribute_definition[1].to_string(),
                                    randomized_data,
                                );
                                break;
                            }
                        }
                    }
                    5 => {
                        /*
                            * This only runs when its a compound attribute with a foreign key
                            * Attribute Definition Is Of The Form:
                            * [key definition][0] [attribute name][1] [compound][2] [(compound attribute)][3] [foreign table][4]
                        */
                        let (referenced_table, referenced_attribute) =
                            get_references(&attribute_definition, 4);

                        if get_referenced_attribute(
                            key_dictionary.get(&referenced_table).unwrap(),
                            &referenced_attribute.to_uppercase().to_string(),
                        )
                            .is_none()
                        {
                            println!(
                                "\nPROGRAM ERROR IN GENERATING DATA [Getting Referenced Attribute]"
                            );
                            std::process::exit(1);
                        }

                        loop {
                            /*
                                * Reference does exist and is valid.
                                * Get a random reference for that attribute from the list
                            */
                            let randomized_data = unique_attribute_checker
                                .get(&referenced_attribute.to_string())
                                .unwrap()
                                .choose(&mut thread_rng())
                                .unwrap()
                                .to_string();

                            if referenced_attributes
                                .contains_key(&referenced_attribute.to_string())
                            {
                                if referenced_attributes
                                    .get(&attribute_definition[4].to_string())
                                    .unwrap()
                                    .contains(&attribute_definition[1].to_string())
                                {
                                    println!("Data being generated for same attribute.");
                                    println!("Program error.");
                                    println!("Exiting program.");

                                    std::process::exit(1);
                                } else {
                                    if referenced_attributes
                                        .get(&attribute_definition[4].to_string())
                                        .unwrap()
                                        .contains(&randomized_data)
                                    {
                                        continue;
                                    }
                                }
                            }

                            /*
                               * True if table is composite keyed
                               * Pair list can't have same data for keyed attributes if it does, regenerate data for current attribute
                            */
                            if pairwise_attribute {
                                if pair_list.contains(&randomized_data) {
                                    continue;
                                }
                                pair_list.push(randomized_data.clone());
                            } else {
                                //Not pairwise, check if data is unique for attribute or continue loop and generate new data for attribute
                                if attribute_definition[0].starts_with("PK")
                                    || attribute_definition[0].starts_with("AK")
                                {
                                    if unique_attribute_checker
                                        .contains_key(&attribute_definition[1])
                                        && unique_attribute_checker[&attribute_definition[1]]
                                        .contains(&randomized_data)
                                    {
                                        continue;
                                    }

                                    unique_attribute_checker
                                        .entry(attribute_definition[1].to_string())
                                        .or_insert(Vec::new())
                                        .push(randomized_data.clone());
                                }
                            }

                            /*
                                * Add the attribute to the list of referenced attributes
                                * Add the data to the statement data
                                * Break out of the loop
                             */
                            referenced_attributes
                                .entry(attribute_definition[4].to_string())
                                .or_insert(Vec::new())
                                .push(attribute_definition[1].to_string());

                            statement_data.insert(
                                attribute_definition[1].to_string(),
                                randomized_data,
                            );
                            break;

                        }
                    }
                    _ => {
                        println!("\nPROGRAM ERROR IN GENERATING DATA [Attribute Definition Length]");
                        std::process::exit(1);
                    }
                }
            }
            /*
                * Check if the table is a pairwise table
                * If it is, check if the pair has been generated before
                * If it has, generate new data for the pair
             */

            if pairwise_table {
                //If pairwise table, make sure composite key is not already existent
                if !unique_pair_checker.contains_key(&table_name) {
                    //True if table does not exist in unique_pair_checker
                    unique_pair_checker
                        .entry(table_name.clone())
                        .or_insert(Vec::new())
                        .push(pair_list.clone());
                } else {
                    // Composite table exists, check if pair has been generated before
                    let (pair_changed, new_pair) = check_pair(
                        &pair_list,
                        &unique_pair_checker[&table_name],
                        &attribute_order_keeper,
                        &unique_attribute_checker,
                        0,
                    );
                    if pair_changed {
                        // Pair did exist and new data was generated in check_pair, change data in statement_data to match
                        // Rewrite the data for the composite key attributes in the statement data
                        let mut index = 0; // Used to match the new data with the correct attribute
                        for attribute in attribute_order_keeper.iter() {
                            let attribute_definitions: Vec<&str> =
                                attribute.split_ascii_whitespace().collect();

                            if primary_keys.contains(&attribute.to_string()) {
                                // Attribute is part of composite key
                                // Rewrite the data for the attribute
                                statement_data.insert(
                                    attribute_definitions[1].to_string(),
                                    new_pair[index].to_string(),
                                );
                                index += 1;
                            }
                        }
                    }

                    // Add the new pair to the unique_pair_checker
                    unique_pair_checker
                        .entry(table_name.clone())
                        .or_insert(Vec::new())
                        .push(new_pair.clone());
                }
            }

            // Write the insert statement to the file. Executes each time, will modify later for improved time complexity
            match write_to_file(
                create_insert_statement(&table_name, &table_attributes, &statement_data),
                path,
            ) {
                Ok(_) => {
                    // Successfully wrote to file, increment statements_generated and print progress
                    statements_generated += 1;
                    print!(
                        "\rGenerating SQL Inserts: {}/{} Created",
                        statements_generated, iterations
                    );
                    stdout().flush().unwrap();
                }
                Err(_) => {
                    // Usually implies bigger error, just exit the program
                    println!("[!] Unable to write to file");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn merge_compound(attribute_definition: &mut Vec<String>) -> bool {
    /*
        * Merges compound attributes into one string
        * Example: 'full_name, COMPOUND, (first_name, VARCHAR(20);, middle_initial CHAR(1):, last_name VARCHAR(20))'
        * Becomes: 'full_name, COMPOUND, (first_name VARCHAR(20); middle_initial CHAR(1); last_name VARCHAR(20))'

        :parameters:
            - `attribute_definition`: The vector of attribute definitions

        :returns:
            - `bool`: True if the attribute definition is a compound attribute, False otherwise
    */
    if attribute_definition.contains(&"compound".to_string())
        || attribute_definition.contains(&"COMPOUND".to_string())
    {
        /*
            * Join All Elements After The Element Containing "COMPOUND" Into One String Element
            * "COMPOUND" Can Either Be At attribute_checker[1] Or attribute_checker[2]
            * Attribute Checker Should Then Have Either 3 Elements Or 4 Elements
            * 3 Elements -> [attribute_name, COMPOUND, compound_attributes]
            * 4 Elements -> [key_definition, attribute_name, COMPOUND, compound_attributes]
            * 5 Elements -> [key_definition, attribute_name, COMPOUND, compound_attributes, foreign_table]
        */
        let mut isReference: bool = false;
        let mut foreign_table: String = String::new();
        let index_of_compound = attribute_definition // Get index of element of string "compound" or "COMPOUND"
            .iter()
            .position(|x| x == "compound" || x == "COMPOUND")
            .unwrap(); // Get index of "compound"

        let index_of_end_compound = if attribute_definition[0].contains("FK") {
            /*
                * Compound Attribute Is A Foreign Key
                * Get Index Of Element Containing "))" (End Of Compound Attribute)
                * This Is Because The Compound Attribute Will Contain The Foreign Table
                * Keeps Foreign Table As Its Own Element
            */
            isReference = true;
            attribute_definition // Get index of element of string ")"
                .iter()
                .position(|x| x.contains("))"))
                .unwrap() // Get index of "))"
        } else {
            /*
                * Not A Foreign Table
                * Proceed As Before
            */
            attribute_definition.len()
        };

        let _compound_attribute = attribute_definition[index_of_compound + 1..index_of_end_compound].join(" "); // Merge compound attr into one string

        /*
            * Remove All Elements After The Element Containing "COMPOUND"
            * Then Push The Compound Attribute To The Attribute Checker
        */
        attribute_definition.truncate(index_of_compound + 1); // Remove excess elements
        attribute_definition.push(_compound_attribute); // Push new compound attribute to end
        if isReference {
            foreign_table = attribute_definition[index_of_end_compound + 1..].join("");
            attribute_definition.push(foreign_table);
        }
        return true;
    }
    false
}

fn set_variable_size(attr_type: &str) -> Option<Vec<u16>> {
    /*
        Improved set_variable_size function
        Still performs the same functionality as the old function
        However this function is able to account for decimal values
        i.e MONEY(10,2) -> 10 is the total number of digits, 2 is the number of digits after the decimal point
        i.e DECIMAL(8,4) -> 8 is the total number of digits, 4 is the number of digits after the decimal point

        :parameters:
            - `attr_type`: The type of the attribute

        :returns:
            - `Option<Vec<u16>>`: The variable size for the attribute returned as a vec
            -  Vec[0] -> Original variable size similar to old function, or number of digits before decimal point
            -  Vec[1] -> Number of digits after decimal point (usually the only reason for index 1 to exist)
            -  None -> No variable size for the attribute
    */
    let some_returned_value: Option<Vec<u16>> = match &attr_type {
        //Check if s.contains("VARIABLE") [in cases of VARIABLE(n)] and s is not "VARIABLE"
        s if s.contains("VARCHAR") && !(*s).eq("VARCHAR") => {
            let variable_size_str = &attr_type[7..];
            let variable_size_str: String =
                variable_size_str[1..variable_size_str.len() - 1].to_string();
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if s.contains("CHAR") && !(*s).eq("CHAR") => {
            let variable_size_str = &attr_type[4..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if (s.contains("PASSWORD") && !(*s).eq("PASSWORD"))
            || (s.contains("USERNAME") && !(*s).eq("USERNAME")) =>
            {
                let variable_size_str = &attr_type[8..];
                let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
                Some(vec![variable_size_str.parse().unwrap(), 0])
            }
        s if s.contains("MONEY") && !(*s).eq("MONEY") => {
            let variable_size_str = &attr_type[5..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if s.contains("DECIMAL") && !(*s).eq("DECIMAL") => {
            let variable_size_str = &attr_type[7..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            let variable_size_str: Vec<&str> = variable_size_str.split(",").collect();
            Some(vec![
                variable_size_str[0].trim().parse().unwrap(),
                variable_size_str[1].trim().parse().unwrap(),
            ])
        }
        _ => None,
    };
    some_returned_value
}

fn write_to_file(generated_statement: String, path: &String) -> io::Result<()> {
    /*
        * Writes the generated statement to the file specified by path variable
        * If the file does not exist, it is created
        * If the file does exist, the generated statement is appended to the file

        :parameters:
            - `generated_statement`: The generated statement to write to the file
            - `path`: The path to write the generated statement to

        :returns:
            - `io::Result<()>`: The result of writing to the file
    */

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .unwrap();

    writeln!(file, "{}", generated_statement)?;

    Ok(())
}

fn main() {
    /*
     * Main function for the program
     * Takes command to input sql tables with their attributes to create mock data
     * Able to produce mock data for multiple tables at once up
     * Capable of Reference Integrity and Unique/Keyed Attributes
     */

    let mut custom_path: Option<String> = None;
    let mut tables: Vec<String> = Vec::new();
    let mut key_dictionary: HashMap<String, Vec<String>> = HashMap::new();
    let mut reference_dictionary: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
    let mut total_iterations: u16 = 0;

    display_help(false);

    loop {
        print!("[*] Manage Tables --> ");
        let _ = stdout().flush();
        let mut sql_input: String = String::new();
        io::stdin()
            .read_line(&mut sql_input)
            .expect("Failed to read SQL command");

        match sql_input.trim().to_lowercase().as_str() {
            // There's at least one table to generate mock data for
            "generate" | "gen" | "run" if !tables.is_empty() => {
                if custom_path.is_none() {
                    let user_folder = dirs::home_dir().unwrap();
                    //Set default path to users documents directory and create/overwrite file named sample-data.sql
                    let documents_dir = user_folder.join("Documents");
                    let file_path = documents_dir.join("sample-data.sql");
                    custom_path = Some(file_path.to_str().unwrap().to_string());
                }

                let custom_path = custom_path.unwrap();
                fs::write(&custom_path, "").expect("Unable to write to file");

                println!("[*] Generating Mock Data...");
                generate_mock_data(
                    &tables,
                    &key_dictionary,
                    &reference_dictionary,
                    &custom_path,
                    total_iterations,
                );

                //Tell user where the file is located
                println!("\n[*] Mock Data Generated In '{}'", custom_path);
                break;
            }
            // No tables to generate mock data for
            "generate" | "gen" | "run" => {
                println!("[!] No Tables to Generate Mock Data For");
                continue;
            }
            // Clear the terminal
            "clear" => {
                print!("{}[2J", 27 as char);
                continue;
            }
            // Display entire help menu
            "help" => {
                display_help(true);
                continue;
            }
            // Exit the program
            "exit" | "quit" => {
                println!(
                    "All tables and mock data will be lost. Are you sure you want to exit? (y/n)"
                );
                let _ = stdout().flush();
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read SQL command");
                if input.trim().to_lowercase().as_str() == "y" {
                    println!("[*] Exiting...");
                    break;
                } else {
                    continue;
                }
            }
            _ => {
                sql_input = sql_input.replace("\\", "\\\\");
                let command_tokens = shlex::split(&sql_input.trim()).unwrap_or_default();
                let sql_command_list: Vec<&str> =
                    command_tokens.iter().map(|x| x.as_str()).collect();
                if sql_command_list.len() < 2 {
                    println!("[!] Invalid SQL Command");
                    continue;
                }

                match sql_command_list[0].to_lowercase().as_str() {
                    "path" => {
                        /*
                            * Get Path From sql_command_list[1]
                            * Then Check If Directory And If The File Already Exists.
                            * If It Does, Ask To Overwrite.
                            * If Not, Create The File
                        */
                        let path: &str = sql_command_list[1].trim();

                        if std::path::Path::new(&path).exists() {
                            println!("[!] File already exists. Overwrite? (y/n)");
                            let _ = stdout().flush();
                            let mut input = String::new();
                            io::stdin()
                                .read_line(&mut input)
                                .expect("Failed to read SQL command");
                            if input.trim().to_lowercase().as_str() == "y" {
                                fs::write(&path, "").expect("Unable to write to file");
                                println!("[*] File overwritten");
                            } else {
                                println!("[*] File not overwritten");
                                continue;
                            }
                        } else {
                            fs::write(&path, "").expect("Unable to write to file");
                            println!("[*] File created");
                        }
                        custom_path = Some(path.to_string());
                    }
                    "add" => {
                        let mut valid: bool = true;
                        let value = sql_command_list[1].parse::<u16>();
                        match value {
                            Ok(iterations) => {
                                if iterations <= 0 {
                                    println!("[!] Invalid Number of Iterations");
                                    valid = false;
                                    break;
                                }
                                total_iterations += iterations;
                            }
                            Err(_) => {
                                println!("[!] Error With Iteration Input");
                                valid = false;
                                break;
                            }
                        };

                        let attributes = sql_command_list[3..sql_command_list.len()].join(" ");
                        let attributes: String = attributes[1..attributes.len() - 1].to_string(); // Remove beginning and ending parentheses
                        let attributes = attributes.split(",").collect::<Vec<&str>>();

                        // Go through each elemeent in the attributes vector and trim it before the for loop below
                        let attributes = attributes
                            .iter()
                            .map(|elem| elem.trim())
                            .collect::<Vec<&str>>();

                        println!("Attributes: {:?}", attributes);

                        for attribute in attributes {
                            let mut isCompound: bool = false;

                            let mut attribute_checker: Vec<String> = attribute
                                .trim()
                                .split_whitespace()
                                .map(|s| s.to_owned())
                                .collect();

                            isCompound = merge_compound(&mut attribute_checker);

                            match attribute_checker.len() {
                                1 => match attribute_checker[0].to_uppercase().as_str() {
                                    "0" | "NULL" | "TRUE" | "FALSE" => {
                                        println!("[!] Valid default value for attribute");
                                    }
                                    _ => {
                                        println!("[!] Invalid default value for attribute");
                                        valid = false;
                                        break;
                                    }
                                },
                                2 => {
                                    if !check_data_type(
                                        attribute_checker[1].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }
                                }
                                3 => {
                                    /*
                                        * Examples Of Possible Attribute Definitions
                                        * PK userID INTEGER
                                        * AK email EMAIL
                                        * full_name COMPOUND (FIRST_NAME VARCHAR(20), MIDDLE_INITIAL CHAR(1), LAST_NAME VARCHAR(20))
                                    */
                                    if !(check_data_type(
                                        //Returns true if attribute is standard attribute
                                        attribute_checker[2].to_uppercase().as_str(),
                                    ) || check_data_type(
                                        //Returns true if compound attribute
                                        attribute_checker[1].to_uppercase().as_str(),
                                    )) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }

                                    if isCompound {
                                        if !check_compound_attribute(2, &attribute_checker) {
                                            /*
                                                * Error In Compound Attribute
                                                * Reason Printed In Function
                                            */
                                            valid = false;
                                            break;
                                        }
                                    } else {
                                        /*
                                         * A compound attribute with a key definition will have a len of 4
                                         * No possible way a compound attribute can have a key definition in this case
                                         * Hence only standard attributes are checked
                                         */
                                        if !check_key_definition(
                                            attribute_checker[0].to_uppercase().as_str(),
                                        ) {
                                            println!("[!] Invalid Key Definition");
                                            valid = false;
                                            break;
                                        }
                                        if attribute_checker[0].to_uppercase() == "PK"
                                            || attribute_checker[0].to_uppercase() == "AK"
                                        {
                                            key_dictionary
                                                .entry(sql_command_list[2].to_string())
                                                .or_insert(Vec::new())
                                                .push(attribute.to_uppercase().to_string());
                                        } else {
                                            println!("Possible Error In Key Definition For Compound Attribute");
                                            println!("Key Definition is Valid. But possible foreign key without proper reference");
                                            valid = false;
                                            break;
                                        }
                                    }
                                }
                                4 => {
                                    /*
                                        * Example Of Compound Attribute With Indices
                                        * [PK/AK][0] MBR[1] COMPOUND[2] (X_MIN; X_MAX; Y_MIN; Y_MAX)[3]
                                    */
                                    if !check_key_definition(
                                        attribute_checker[0].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Key Definition");
                                        valid = false;
                                        break;
                                    }

                                    if !check_data_type(
                                        attribute_checker[2].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }

                                    if isCompound {
                                        if !check_compound_attribute(3, &attribute_checker) {
                                            valid = false;
                                            break;
                                        }

                                        // Is a keyed compound attribute (Either PK or AK) add compound attribute to key dict
                                        if attribute_checker[0].to_uppercase() == "PK"
                                            || attribute_checker[0].to_uppercase() == "AK"
                                        {
                                            key_dictionary
                                                .entry(sql_command_list[2].to_string())
                                                .or_insert(Vec::new())
                                                .push(attribute.to_uppercase().to_string());
                                        } else {
                                            println!("Possible Error In Key Definition For Compound Attribute");
                                            println!("Key Definition is Valid. But possible foreign key without proper reference");
                                            valid = false;
                                            break;
                                        }
                                    } else {
                                        // Just a foreign key reference handle as before
                                        let (referenced_table, referenced_attribute_name) =
                                            get_references(&attribute_checker, 3);

                                        let referenced_attribute = get_referenced_attribute(
                                            &key_dictionary
                                                .get(&referenced_table.to_string())
                                                .unwrap_or(&vec![]),
                                            &referenced_attribute_name.to_uppercase().to_string(),
                                        );

                                        match referenced_attribute {
                                            Some(_) => {
                                                if attribute_checker[0]
                                                    .to_uppercase()
                                                    .contains("PK")
                                                    || attribute_checker[0]
                                                    .to_uppercase()
                                                    .contains("AK")
                                                {
                                                    key_dictionary
                                                        .entry(sql_command_list[2].to_string())
                                                        .or_insert(Vec::new())
                                                        .push(attribute.to_string());
                                                }
                                                reference_dictionary
                                                    .entry(sql_command_list[2].to_string())
                                                    .or_insert(Vec::new())
                                                    .push(HashMap::from([(
                                                        referenced_table.to_string(),
                                                        referenced_attribute_name.to_string(),
                                                    )]));
                                            }
                                            None => {
                                                println!("[!] Invalid Reference");
                                                valid = false;
                                                break;
                                            }
                                        }
                                    }
                                }
                                5 => {
                                    /*
                                     * This will ONLY run with a foreign key compound attribute
                                     * FK full_name COMPOUND (FIRST_NAME VARCHAR(20), MIDDLE_INITIAL CHAR(1), LAST_NAME VARCHAR(20)) profile(full_name)
                                     */
                                    if !check_key_definition(
                                        attribute_checker[0].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Key Definition");
                                        valid = false;
                                        break;
                                    }

                                    if !check_data_type(
                                        attribute_checker[2].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }

                                    if !check_compound_attribute(3, &attribute_checker) {
                                        valid = false;
                                        break;
                                    }

                                    // Compound Attribute With Foreign Key Reference
                                    let (referenced_table, referenced_attribute_name) =
                                        get_references(&attribute_checker, 4);

                                    let referenced_attribute = get_referenced_attribute(
                                        &key_dictionary
                                            .get(&referenced_table.to_string())
                                            .unwrap_or(&vec![]),
                                        &referenced_attribute_name.to_uppercase().to_string(),
                                    );

                                    match referenced_attribute {
                                        Some(_) => {
                                            if attribute_checker[0].to_uppercase().contains("PK")
                                                || attribute_checker[0]
                                                .to_uppercase()
                                                .contains("AK")
                                            {
                                                key_dictionary
                                                    .entry(sql_command_list[2].to_string())
                                                    .or_insert(Vec::new())
                                                    .push(attribute.to_string());
                                            }
                                            reference_dictionary
                                                .entry(sql_command_list[2].to_string())
                                                .or_insert(Vec::new())
                                                .push(HashMap::from([(
                                                    referenced_table.to_string(),
                                                    referenced_attribute_name.to_string(),
                                                )]));
                                        }
                                        None => {
                                            println!("[!] Invalid Reference");
                                            valid = false;
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    println!("Error In Attributes For The Table: {}...Attribute: {} Is Not Formatted Properly", sql_command_list[2], attribute);
                                    valid = false;
                                    break;
                                }
                            }
                        }
                        if valid {
                            let table_string =
                                sql_command_list[1..sql_command_list.len()].join(" ");
                            tables.push(table_string);
                            println!(
                                "[*] {} Insert Statements Added For {}",
                                sql_command_list[1], sql_command_list[2]
                            );
                        } else {
                            total_iterations -= sql_command_list[1].parse::<u16>().unwrap();
                            continue;
                        }
                    }
                    "remove" | "rm" | "del" => {
                        let table_to_delete = tables
                            .iter()
                            .find(|elem| {
                                elem.split_whitespace().nth(1) == Some(sql_command_list[1])
                            })
                            .cloned();
                        if let Some(table) = table_to_delete {
                            let num_statements = table.split_whitespace().nth(0).unwrap();
                            total_iterations -= num_statements.parse::<u16>().unwrap();
                            tables.retain(|elem| elem != &table);
                        } else {
                            println!("[!] Table Not Found");
                        }
                    }
                    "modify" | "mod" => {
                        let _table_to_modify = tables
                            .iter()
                            .find(|elem| {
                                elem.split_whitespace().nth(1) == Some(sql_command_list[1])
                            })
                            .cloned();
                        // Find attribute name at sql_command_list[2] and replace the entire attribute with sql_command_list[3..sql_command_list.len()]
                        // TODO
                    }
                    "show" => {
                        // If len == 3 Then A Specific Table Has Been Given To Pull Data From
                        // Else
                        let specifier = if sql_command_list.len() == 3 {
                            Some(sql_command_list[2].to_lowercase())
                        } else {
                            None
                        };

                        match sql_command_list[1].to_lowercase().as_str() {
                            // Show Every Table The Program Will Generate Inserts For [Unless Given A Specific Table]
                            "inserts" => {
                                for table in &tables {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                        == table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        println!(
                                            "Table -> {}: Associated Tuple -> {}",
                                            table.split(' ').nth(1).unwrap(),
                                            table
                                        );
                                    }
                                }
                            }
                            // Show The Keys For Every Table [Unless Given A Specific Table]
                            "keys" => {
                                for (table, keys) in &key_dictionary {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                        == table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        println!("Keys For Table {} -> {:?}", table, keys);
                                    }
                                }
                            }
                            // Show's All Tables That Reference Another Table [Unless Given A Specific Table]
                            "references" | "refs" => {
                                for (ref_table, key) in &reference_dictionary {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                        == ref_table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        for value in key {
                                            for (r_key, r_value) in value {
                                                println!("Referencing Table -> {} References {} From Table {}", ref_table, r_value, r_key);
                                            }
                                        }
                                    }
                                }
                            }
                            // Show's All Custom Types Made Specifically For This Program
                            "types" => {
                                println!(
                                    "
                                        Custom Types Created For This Program

                                        NAME -> Name is a custom type used to replace typical string DataTypes in sql for
                                        when the user wishes to generate a realistic name, assigning a typical
                                        VARCHAR(length) to an attribute where you wish to be a name may result in a randomly
                                        generated string that does not represent a persons full name in real life

                                        EMAIL -> Email is another custom type used to replace string DataTypes in sql. This
                                        ensures that every value generated for said attribute will follow standard email
                                        format 'abc@abc.abc' and also excludes symbols that are commonly excluded in
                                        deployed email domain services (^&% etc) 

                                        PASSWORD(N) -> Password is another custom type used to replace string DataTypes in sql.
                                        While it generates similar strings that would be generated should you just do a
                                        VARCHAR(length) variable in its place, PASSWORD types have the added step of 
                                        ensuring common password complexity constraints are enforced such as
                                            Password Must Include At Least: 
                                                [1 Capital Letter, 1 Lowercase, 1 Number, 1 Special Char]
                                                [N = Max Length Of Password And It Generates Length Between 8..N]
                                                [NOTE] IF YOU DO NOT ADD A LENGTH TO PASSWORD I.E PASSWORD(40)
                                                [THEN] THE PROGRAM WILL PANIC. THIS INCLUDES STANDARD VARCHAR(N) AND CHAR(N) TYPES

                                        GROUP -> Is a custom DataType that technically would replace String SQL types.
                                        Assigning this type to an attribute will assign either 'mod' or 'member' as a role

                                        USERNAME(N) -> Is a custom DataType that technically would replace String SQL types.
                                        Assigning this type to an attribute will assign a randomly generated username

                                        MONEY(N) -> Is a shorthand for designating decimal values of xxx.xx
                                        All values will only have 2 decimal places between 0..99
                                        N -> Max figure for the money value (i.e MONEY(7) generates values between 0.00 and 9,999,999.99)

                                        ## NUMEROUS OTHER TYPES HAVE BEEN ADDED THAT I HAVE YET TO ADD DESCRIPTIONS FOR ##

                                        COMPOUND -> Is a custom DataType that is used to designate a compound attribute
                                        Compound Attributes Are Attributes That Are Made Up Of Multiple Attributes
                                        Compound Attributes Must Be Defined In The Following Format:
                                        [key definition] [attribute name] COMPOUND ([attribute 1]; [attribute 2]; ...; [attribute n])
                                        Where [key definition] Can Be Null

                                        [*] For More Help With Attributes, Type 'show examples attributes'
                                        [*] For More Help With Compound Attributes, Type 'show examples compound'
                                    "
                                );
                            }
                            // Show's Examples Of Commands To Aid The User Optional [Specifier] Shows Only Specific Examples
                            // If Specifier Not Given. All Examples Are Shown
                            "examples" | "ex" => {
                                // Show's Examples Of How To Add Tables
                                if specifier.is_none() || specifier == Some("add".to_owned()) {
                                    // Examples For Add
                                    println!("
                                        Add Example [Adding 'profile' Table To List]:
                                        [Add Example #1] -> add 100 profile (PK userID INTEGER, name NAME, AK email EMAIL, password PASSWORD(30), dateOfBirth DATE, lastLogin TIMESTAMP)
                                        [Add Example #2] -> add 300 friend (PK/FK userID1 INTEGER profile(userID), PK/FK userID2 INTEGER profile(userID), friendDate DATE)
                                        [Add Example #3] -> add 1000 post (PK postID INTEGER, FK userID INTEGER profile(userID), postDate TIMESTAMP, postContent VARCHAR(1000))
                                        [Add Example #4] -> add 1000 comment (PK commentID INTEGER, FK userID INTEGER profile(userID), FK postID INTEGER post(postID), commentDate TIMESTAMP, commentContent VARCHAR(1000))
                                        [Add Example #5] -> add 1000 MBR (PK MBR_ID INTEGER, MBR COMPOUND (X_MIN INTEGER, X_MAX INTEGER, Y_MIN INTEGER, Y_MAX INTEGER))
                                    ");
                                }
                                // Show's Examples Of How To Delete Tables
                                if specifier.is_none() || specifier == Some("del".to_owned()) {
                                    // Examples For Del
                                    println!(
                                        "
                                        Rm Example [Assuming 'profile' Table Is Defined]:
                                        [Rm Example #1] -> Rm profile
                                    "
                                    );
                                }
                                // Show's Examples Of How To Modify Tables
                                if specifier.is_none()
                                    || specifier == Some("modify".to_owned())
                                    || specifier == Some("mod".to_owned())
                                {
                                    // Examples For Modify
                                    println!(
                                        "
                                        Modify Example [Assuming 'profile' Table Is Defined From 'Add' Example]:
                                        [In This Case This Example Converts The PK 'email' Attribute To A AK]
                                        
                                        [Current Attribute Definition] -> PK email EMAIL
                                        [Mod Example #1] -> mod profile email AK email EMAIL
                                        [New Attribute Definition] -> AK email EMAIL

                                        [NOTE] The Attribute Definition Must Be In The Same Format As The Original
                                        "
                                    );
                                }
                                // Explains Reference Formatting To Aid In Table Input
                                if specifier.is_none()
                                    || specifier == Some("references".to_owned())
                                    || specifier == Some("refs".to_owned())
                                {
                                    println!(
                                        "
                                        Attribute Format Explained:
                                        All Attributes In A Table Definition Must Be Of The Form:
                                        '[key definition][attribute name][attribute type][foreign table]'
                                        Where [key definition] And [foreign table] Can Be Null

                                        If [key definition] Is A Foreign Key [FK | PK/FK | AK/FK]:
                                            Then [foreign table] MUST be defined

                                        The Following Is An Example PK/FK Attribute
                                        Example References A Table Named 'profile' And Its Attribute 'userID'
                                        Attribute -> 'PK/FK userID1 INTEGER profile(userID)'
                                        The Example Would Assign As So:
                                            [key definition] = 'PK/FK'
                                                Is A Primary Key That References Primary Key 'userID' From Table 'profile'
                                            [attribute name] = 'userID1'
                                                Self Explanatory, It Is The Name Of The Attribute Being Defined
                                            [attribute type] = 'INTEGER' 
                                                Type Definitions On Each Attribute Must Match SQL DataTypes, Not Pythons
                                            [foreign table] = 'profile(userID)' Where:
                                                [*] 'profile' Is The Table Being Referenced
                                                [*] 'userID' Is The Referenced Attribute
                                        [*] For More Help With Attributes, Type 'show examples attributes'

                                        In Tables With More Than One PK, It Will Generate Data As A Composite Pair,
                                        Hence The AK Attribute May Be Needed, This Is Enforces The Unique Value Without
                                        Worrying About Checking The Keys As Pairs

                                        In Cases With Tables Having Multiple Attributes: 
                                        They Must Be Placed In () And Separated By Commas.
                                        'show examples add' Provides Examples
                                        "
                                    );
                                }
                                // Explains Attribute Formatting To Aid In Table Input
                                if specifier.is_none()
                                    || specifier == Some("attributes".to_owned())
                                    || specifier == Some("attr".to_owned())
                                {
                                    println!(
                                        "
                                            Basic Introductory To Attribute Types For SQL Tables
                                        ---------------------------------------------------------------------------
                                        [*] To See Custom Defined Types Specifically For This Program
                                        [*] Type 'show examples types'

                                        [Example 1]: 'age INTEGER'
                                        [key definition] = None
                                            [*] No Key Definition is Included
                                        [attribute name] = 'age'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types Not Rust Data Types

                                        [Example 2]: 'salary DECIMAL(10,2)'
                                        [key definition] = None
                                            [*] 'No Key Definition is Included
                                        [attribute name] = 'salary'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'DECIMAL(10,2)'
                                            [*] Must Be SQL Data Types Not Rust Data Types

                                        [Example 3]: 'name NAME'
                                        [key definition] = None
                                            [*] No Key Definition is Included
                                        [attribute name] = 'orderID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'NAME'
                                            [*] Must Be SQL Data Types Not Rust Data Types
                                            [*] NAME Is A Custom Type Made Specifically For This Program
                                            [*] NAME Type Ensures A Realistic Full Name Is Generated
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'

                                        [Example 4]: 'PK userID INTEGER'
                                        [*] Is a Primary Key Attribute
                                        [key definition] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name] = 'userID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types

                                        [Example 5]: 'AK email EMAIL'
                                        [*] Is a Alternate Key Attribute
                                        [key definition] = 'AK'
                                            [*] 'AK' stands for Alternate Key
                                        [attribute name] = 'userID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'EMAIL'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                            [*] EMAIL Is A Custom Type Made Specifically For This Program
                                            [*] EMAIL Type Ensures A Realistic Email Is Generated
                                            [*] If A Name Attribute Is Present In The Same Table And Data Gets Generated For It Before Email
                                            [*] The Email Generated Will Be Representative Of The Name Generated As Well
                                            [*] Example If A Name 'Bob Johnson' Is Generated For A Tuple:
                                            [*] Then A Possible Email Generated Would Be BobJohnson@outlook.com
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'

                                        [Example 6]: 'FK customerID INTEGER customers(customerID)'
                                        [*] Is a Foreign Key Attribute Referencing the 'customers' Table's 'customerID' Attribute
                                        [key definition] = 'FK'
                                            [*] 'FK' stands for Foreign Key
                                        [attribute name] = 'customerID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types

                                        [Example 7]: 'AK/FK customerID INTEGER references customers(customerID)'
                                        [*] Is an Alternate Key and Foreign Key Attribute References the 'customers' Table's 'customerID' Attribute
                                        [key definition] = 'AK/FK'
                                            [*] 'AK/FK' stands for Alternate Key/Foreign Key
                                        [attribute name] = 'customerID'
                                            [*] Name of Attribute Being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types

                                        [Example 8]: 'PK/FK bookID INTEGER authors(bookID)'
                                        [*] Is a Primary Key and Foreign Key Attribute Referencing the 'authors' Table's 'bookID' Attribute
                                        [key definition] = 'PK/FK'
                                            [*] 'PK' stands for Primary Key
                                            [*] 'FK' stands for Foreign Key
                                            [*] 'PK/FK' stands for Primary Key with a Foreign Reference
                                        [attribute name] = 'bookID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [foreign table] = 'authors(bookID)'
                                            [*] Where Again, 'authors' Is The Table Being Referenced and 'bookID' Is The Referenced Attribute From The Table

                                        [Example 9]: 'PK userID INTEGER, PK postID INTEGER
                                        [*] These Primary Key Attributes Create A Composite Key
                                        [key definition] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name 1] = 'userID'
                                            [*] Name of the First Attribute Being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must be SQL Data Types, Not Rust Data Types
                                        [attribute name 2] = 'postID'
                                            [*] Name of the second Attribute Being Defined
                                        [attribute type 2] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                        [*] Both 'userID' and 'postID' are Primary Keys
                                        [*] This Means Their Combination Will Be Treated As A Composite Key Pair When Generating Random Data For The Table. 
                                        [*] This Means That Any Two Rows On The Table Cannot Have The Same Combination of 'userID' and 'postID'.

                                        [Example 10]: 'PK/FK bookID INTEGER authors(bookID), PK/FK publisherID INTEGER publishers(publisherID)'
                                        [*] These Primary Key Attributes Create A Composite Key Where Each Attribute Also References An Exterior Table
                                        [key definition] = 'PK/FK'
                                            [*] 'PK' stands for Primary Key
                                            [*] 'FK' stands for Foreign Key
                                            [*] 'PK/FK' stands for Primary Key with a Foreign Reference
                                        [attribute name 1] = 'bookID'
                                            [*] Name of the First Attribute being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [attribute name 2] = 'publisherID'
                                            [*] Name of the Second Attribute being Defined
                                        [attribute type 2] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [foreign table] = 'authors(bookID), publishers(publisherID)'
                                            [*] Where Again, 'authors' Is The Table Being Referenced and 'bookID' Is The Referenced Attribute From The Table
                                            [*] As Well As, 'publisher' Is The Table Being Referenced and 'publisherID' Is The Referenced Attribute From The Table
                                            [*] Both 'userID' and 'postID' are Primary Keys
                                            [*] This Means Their Combination Will Be Treated As A Composite Key Pair When Generating Random Data For The Table. 
                                            [*] This Means That Any Two Rows On The Table Cannot Have The Same Combination of 'userID' and 'postID'.

                                        [Example 11]: 'PK userID INTEGER, AK email EMAIL'
                                        [key definition 1] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name 1] = 'userID'
                                            [*] Name of Attribute Being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                        [key definition 2] = 'AK'
                                            [*] 'AK' stands for Alternate Key
                                        [attribute name 2] = 'email'
                                            [*] Name of Attribute Being Defined
                                        [attribute type 2] = 'EMAIL'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                            [*] EMAIL Is A Custom Type Made Specifically For This Program
                                            [*] EMAIL Type Ensures A Realistic Email Is Generated
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'
                                            [*] In This table, The 'userID' Attribute Is A Primary Key And The 'email' Attribute Is An Alternate Key.
                                            [*] While Having Multiple Primary Keys In A Table Leads To A Composite Key Pair.
                                            [*] Having A Primary Key And An Alternate Key In The Same Table Doesn't Necessarily Result In A Composite Key.
                                            [*] The Alternate Key May Still Provide A Unique Constraint, But It Doesn't Have The Same Significance As The Primary Key.
                                        "
                                    );
                                }
                                // Add examples for compound attributes
                                if specifier.is_none()
                                    || specifier == Some("compound".to_owned())
                                    || specifier == Some("comp".to_owned())
                                {
                                    println!(
                                        "
                                            Compound Attribute Format Explained:

                                            Compound Attributes Are Attributes That Are Defined As A Group Of Attributes
                                            That Are All Treated As One Attribute When Generating Data For The Table
                                            For Example. If You Have A Table User That Stores A Users FulL Name As A Compound Attribute
                                            You Would Define It As So:
                                            [Example 1]: 'full_name COMPOUND (FIRST_NAME VARCHAR(20); MIDDLE_INITIAL CHAR(1); LAST_NAME VARCHAR(20))'
                                            [**IMPORTANT NOTE**]: Compound Attributes Must Be Defined In '()' And Separated By ';'
                                            Compound Attributes Can Be Keyless, Have A Key Definition, Or Have A Key Definition With A Foreign Key Reference
                                            Here Are Some Examples:
                                            [Example 2]: 'AK MBR COMPOUND (X_MIN INTEGER; X_MAX INTEGER; Y_MIN INTEGER; Y_MAX INTEGER)'
                                            [Example 3]: 'PK/FK MBR COMPOUND (X_MIN INTEGER; X_MAX INTEGER; Y_MIN INTEGER; Y_MAX INTEGER) region(MBR)'
                                            [Example 4]: 'PK MBR COMPOUND (X_MIN INTEGER; X_MAX INTEGER; Y_MIN INTEGER; Y_MAX INTEGER)'
                                        "
                                    )
                                }
                            }
                            _ => {
                                println!("Invalid Argument For Show Command");
                                println!("Usage: Show [inserts | keys | references | examples [optional - [add | del | (modify | mod) | (refs | references) | (attributes | attr)]]");
                            }
                        }
                    }
                    _ => {
                        println!("Invalid Command Entered");
                        println!("Type 'Help' For List of Commands");
                    }
                }
            }
        }
    }
}

fn display_help(display_all: bool) {
    /*
     * Displays the help menu
     * If display_all is true, display the entire help menu
     * If display_all is false, display the help menu for the generate command
     */
    println!(
        "
                        ===== MESSAGE =====

            [*]This Program Takes User Input For SQL Table Values And Generates Random Insert Statements For Database Testing
            [*]All Tables Should Be Defined Using The Following Format:
                    Format: [Number Of Insert Statements][Table Name][Table Attributes]
            [*]An Example Of An Accepted Table Definition And An Accepted Add Command:
            [*]add 100 profile (PK userID INTEGER, name NAME, AK email EMAIL, password PASSWORD(30))
            [*]Where 'add' Adds The [Number Of Insert Statements][Table Name][Table Attributes] To A List Used For Generation
            [*]This Program Can Generate Any Number Of Inserts For Any Number Of SQL Tables As Long As User Memory Permits
            [*]The Commands For This Program Are Not Case Sensitive
            [*]Type 'Help' To Get A List Of All Commands And To Reprint This Message
            [*]To See Example Commands For Each Command:
                    Type 'show examples [specific command (optional - will show all examples if blank)]'
            [*]Type 'Clear' To Delete This Message
            \n"
    );

    if display_all {
        println!(
            "
                        ===== COMMANDS AND DEFINITIONS=====

            Commands Followed By [] Are Required Args Unless Specifically Stated In This Menu

            Generate -> Begins Generating SQL Script Of Inserts [Mut Have At Least One Defined Table]
            Clear -> Clear Terminal Screen
            Help -> Show This Help Menu
            Exit | Quit -> Terminate Program

            Add  [numInserts][tableName][tableAttributes] -> Add Table To Generate Statements For, Where:
                 [numInserts] -> The Number Of Insert Statements To Generate For The Table
                 [tableName] -> The Name of The Table Being Created
                 [tableAttributes] -> All Defined Columns Of The Table [Format Shown Below]
                 [tableAttributes] Should Be Input In The Following Format:

                 [key definition][referenced_attr name][referenced_attr type][foreign table]

                 Where [key definition] and [foreign table] can be NULL
                 Key Definitions:
                    'PK' - Primary Key
                    'AK' - Alternate (Unique) Key
                    'FK' - Foreign Key -> [foreign table] - Table Being Referenced By The Attribute
                    'PK/FK' - Primary Key And Foreign Key -> [foreign table] - Table Being Referenced By The Attribute
                    'AK/FK' - Alt (Unique) And Foreign Key -> [foreign table] - Table Being Referenced By The Attribute

                 [Note 1]: In Cases Of An Attribute Being A Foreign Key, [foreign table] CANNOT be NULL
                 To Ensure A Foreign Key Attribute Is Properly Read And Referenced A Specific Format Is Provided:
                 [Foreign Table] Proper Format -> referenced_table(referenced_attribute)
                 For More Info On Referencing Syntax With Examples. Input 'show examples [refs | references]'

                 [Note 2]: In Tables With More Than One PK, It Will Generate Data As A Composite Pair
                 Hence The AK Attribute May Be Needed, This Enforces The Unique Values Without Worrying About Checking The Keys As Pairs

            Rm   [tableName] -> Remove Table From List
                 [tableName] Must Be The Same As It Was Defined In It's Add Statement

            Modify | Mod [tableName] [numStatements | tableName | referenced_attr] [newValue] -> Modify Existing Table
                 [tableName] -> The Table To Modify And Must Be The Same As It Was Defined In It's Add Statement
                 [numInserts | tableName | referenced_attr] -> The Definition Of The Tuple You Wish To Modify 
                 Note: You Can Only Modify One Definition At A Time
                 IMPORTANT REGARDING Modify [numInserts | tableName]:
                 If User Wishes To Modify The Number of Statements or Change Table Name. The User Should Input 'numStatements'
                 or 'tableName' in Place of First Arg After Specifying The Table: However, When Modifying [referenced_attr],
                 The User Should Input the Attribute Name They Wish To Change. Type 'show examples mod' For More Info
                 [*]For 'Modify [tableName] [referenced_attr]':
                    The User Must Redefine The Whole Attribute 
                    As In Redefine [key definition][referenced_attr name][referenced_attr type][foreign table]

            Show [inserts | keys | references | types | examples [Add | Del | (Modify | Mod) | (Refs | References) | (Attributes | Attr)]
                 [Inserts] -> Show's The Table's The Program Will Be Creating Insert Statements For
                 [Keys] -> Show's The List Of Keys For Each Table
                 [References | Refs] -> Show's All Referenced Attributes Between Two Tables In The Form Of:
                                        ([Referencing Table]:([Referenced Table]:[Referenced Attribute]))
                 [Types] -> Lists All Custom Defined Types And The Reasoning For Their Creation For This Program 
                 [Examples] -> Will Print Examples For Most Commands Taken By The Program
            "
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{stdout, Write},
    };

    use rand::Rng;

    #[test]
    fn test_attribute_datatype() {
        /*
        Test Valid and Invalid Data Types
        Assert Valid Data Types Return True
        Assert Invalid Data Types Return False
        */

        let valid_datatype = vec!["INTEGER", "VARCHAR(30)", "PASSWORD(30)"];
        let invalid_datatype = vec!["INVALID!", "VAFCHAR", "EMMAIL"];
        for valid in valid_datatype {
            assert_eq!(super::check_data_type(valid), true);
        }
        for invalid in invalid_datatype {
            assert_eq!(super::check_data_type(invalid), false);
        }
    }

    #[test]
    fn test_key_definitions() {
        /*
        Test Valid and Invalid Key Definitions
        For Valid, Test All Possible Key Definitions
        For Invalid, Test Spelling Errors and Incorrect Ordering
        */
        let valid_key_definitions = vec!["PK", "AK", "FK", "PK/FK", "AK/FK"];
        let invalid_key_definitions = vec![
            "KP", "KA", "KF", "PK/AK", "AK/PK", "FK/PK", "FK/AK", "PK/AK/FK",
        ];
        for valid in valid_key_definitions {
            assert_eq!(super::check_key_definition(valid), true);
        }
        for invalid in invalid_key_definitions {
            assert_eq!(super::check_key_definition(invalid), false);
        }
    }

    #[test]
    fn test_random_data_generation() {
        /*
        Test Data Generation With Types With Given Size
        PASSWORD(20), USERNAME(10), MONEY(7), VARCHAR(30)
        Assert Length of Return > 0 and <= Given Size
        */
        let statement_data: HashMap<String, String> = HashMap::new(); //Not Needed For This Test (Only Used For Email Generation)

        let password = super::get_random_data("PASSWORD(20)", Some(vec![20, 0]), &statement_data);
        assert_eq!(password.len() > 0 && password.len() <= 20, true);

        let username = super::get_random_data("USERNAME(10)", Some(vec![10, 0]), &statement_data);
        assert_eq!(username.len() > 0 && username.len() <= 10, true);

        let money = super::get_random_data("MONEY(7)", Some(vec![7, 0]), &statement_data);
        //Since money is returned as String and .{}{} (Used for cents) takes up 3 chars. The length of the return should be 7 + 3 = 10
        assert_eq!(money.len() > 0 && money.len() <= 10, true);

        let varchar = super::get_random_data("VARCHAR(30)", Some(vec![30, 0]), &statement_data);
        assert_eq!(varchar.len() > 0 && varchar.len() <= 30, true);
    }

    #[test]
    fn test_email_generation_with_names_in_statement_data() {
        /*
        Test Email Generation With Names In Statement Data
        Assert Email Contains Name
        This Tests The 3 Given Key Possibilities (name, full name, full_name)
        */
        let mut statement_data: HashMap<String, String> = HashMap::new();
        statement_data.insert("name".to_string(), "Bob Johnson".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("BobJohnson"), true);

        //Reset HashMap and generate new name, then try again
        statement_data = HashMap::new();
        statement_data.insert("full name".to_string(), "John Smith".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("JohnSmith"), true);

        //Reset HashMap and generate new name, then try again
        statement_data = HashMap::new();
        statement_data.insert("full_name".to_string(), "Jane Doe".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("JaneDoe"), true);
    }

    #[test]
    fn test_set_variable_size() {
        /*
        Pass datatype of format DATATYPE(n)
        Assert n is returned
        */
        let data_types = vec![
            "VARCHAR(30)",
            "PASSWORD(20)",
            "USERNAME(10)",
            "MONEY(7)",
            "DECIMAL(10,2)",
        ];
        let returned_types = vec![
            vec![30, 0],
            vec![20, 0],
            vec![10, 0],
            vec![7, 0],
            vec![10, 2],
        ];

        for (index, data_type) in data_types.iter().enumerate() {
            assert_eq!(super::set_variable_size(data_type).is_some(), true);
            assert_eq!(
                super::set_variable_size(data_type).unwrap(),
                returned_types[index]
            );
        }
    }

    #[test]
    fn test_create_insert_statement() {
        /*
        Create vars for function and test returned insert statement
        */
        let target_insert_statement =
            "INSERT INTO profile VALUES (1, 'Bob Johnson', 'BobJohnson@pitt.edu');";
        let table_name = "profile";
        let table_attributes: Vec<String> = vec![
            "userID".to_string(),
            "name".to_string(),
            "email".to_string(),
        ];
        let mut statement_data: HashMap<String, String> = HashMap::new();
        statement_data.insert("userID".to_string(), "1".to_string());
        statement_data.insert("name".to_string(), "Bob Johnson".to_string());
        statement_data.insert("email".to_string(), "BobJohnson@pitt.edu".to_string());

        let generated_insert =
            super::create_insert_statement(table_name, &table_attributes, &statement_data);
        assert_eq!(generated_insert, target_insert_statement);
    }

    #[test]
    fn test_check_pair_with_unique_pair_passed() {
        /*
        Test check_pair with a unique pair
        pair_changed should return false
        Generate values for
                generated_pair_vector: &Vec<String>,
                previous_pairs: &Vec<Vec<String>>,
                table_attributes: &Vec<String>,
                uq_attributes: &HashMap<String, Vec<String>>,
                count: usize,
        */
        let generated_pair_vector: Vec<String> = vec!["1".to_string(), "Bob Johnson".to_string()];
        let previous_pairs: Vec<Vec<String>> = vec![
            vec!["2".to_string(), "John Smith".to_string()],
            vec!["3".to_string(), "Jane Doe".to_string()],
            vec!["4".to_string(), "Steven Even".to_string()],
        ];
        let table_attributes: Vec<String> = vec!["userID".to_string(), "name".to_string()];
        let mut uq_attributes: HashMap<String, Vec<String>> = HashMap::new();
        uq_attributes.insert(
            "userID".to_string(),
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
        );
        uq_attributes.insert(
            "name".to_string(),
            vec![
                "Bob Johnson".to_string(),
                "John Smith".to_string(),
                "Jane Doe".to_string(),
                "Steven Even".to_string(),
            ],
        );
        let count: usize = 0;
        let (pair_changed, _new_pair) = super::check_pair(
            &generated_pair_vector,
            &previous_pairs,
            &table_attributes,
            &uq_attributes,
            count,
        );
        assert_eq!(pair_changed, false);
    }

    #[test]
    fn test_check_pair_with_prev_pair_passed() {
        /*
        Test check_pair with a prev generated pair
        pair_changed should return true
        Then check_pair with new values should return false
        Generate values for
                generated_pair_vector: &Vec<String>,
                previous_pairs: &Vec<Vec<String>>,
                table_attributes: &Vec<String>,
                uq_attributes: &HashMap<String, Vec<String>>,
                count: usize,
        */
        let generated_pair_vector: Vec<String> = vec!["1".to_string(), "Bob Johnson".to_string()];
        let previous_pairs: Vec<Vec<String>> = vec![
            vec!["2".to_string(), "John Smith".to_string()],
            vec!["1".to_string(), "Bob Johnson".to_string()],
            vec!["4".to_string(), "Steven Even".to_string()],
        ];
        let table_attributes: Vec<String> = vec!["userID".to_string(), "name".to_string()];
        let mut uq_attributes: HashMap<String, Vec<String>> = HashMap::new();

        //Add extra padding to both attributes to allow function to generate new pair. Run check pair twice
        uq_attributes.insert(
            "userID".to_string(),
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
            ],
        );
        uq_attributes.insert(
            "name".to_string(),
            vec![
                "Bob Johnson".to_string(),
                "John Smith".to_string(),
                "Jane Doe".to_string(),
                "Steven Even".to_string(),
                "Zoe Tae".to_string(),
                "Jenny Doe".to_string(),
            ],
        );
        let count: usize = 0;
        let (pair_changed, new_pair) = super::check_pair(
            &generated_pair_vector,
            &previous_pairs,
            &table_attributes,
            &uq_attributes,
            count,
        );
        assert_eq!(pair_changed, true);
        let (pair_changed, _new_pair) = super::check_pair(
            &new_pair,
            &previous_pairs,
            &table_attributes,
            &uq_attributes,
            count,
        );
        assert_eq!(pair_changed, false);
    }

    #[test]
    fn test_cast_generated_decimal_to_float() {
        /*
        Using DECIMAL(15, 10) as datatype
        Pass it through check_data_type to confirm exists
        If true pass it through set_variable_size to get size
        assert it returns Some(vec![15, 10])
        Have it generate a value
        Assert that its parsable to f64
        Assert that number of decimal places is 10 and number of digits before decimal is 15
        */
        let mut i = 0;
        let statement_data: HashMap<String, String> = HashMap::new(); //Not Needed For This Test (Only Used For Email Generation)
        while i < 500000 {
            //Create DECIMAL(m, n) Data type where m and n are values within f64 range
            let mut rng = rand::thread_rng();
            let m: u16 = rng.gen_range(1..16);
            let n: u16 = rng.gen_range(1..11);
            let data_type = format!("DECIMAL({}, {})", m, n);
            let data_type = data_type.as_str();
            assert_eq!(super::check_data_type(data_type), true);
            let size = super::set_variable_size(data_type);
            assert_eq!(size.is_some(), true, "Returned a None Value");
            assert_eq!(
                size.clone().unwrap(),
                vec![m, n],
                "Size not equal to vec![15, 10]"
            );
            let generated_value = super::get_random_data(data_type, size, &statement_data);
            assert_eq!(
                generated_value.parse::<f64>().is_ok(),
                true,
                "Failed To Parse Generated Decimal Value To f64"
            );
            let decimal_split: Vec<&str> = generated_value.split('.').collect();
            assert!(
                decimal_split[0].len() <= m as usize,
                "Number Of Digits Before Decimal Is Greater Than {}",
                m
            );
            assert!(
                decimal_split[1].len() <= n as usize,
                "Number Of Digits After Decimal Is Greater Than {}",
                n
            );
            i += 1;
            print!("\rIteration {} Passed", i);
            stdout().flush().unwrap();
        }
        println!();
    }
}
