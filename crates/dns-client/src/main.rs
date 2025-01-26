use dns::protocol::answer::AnswerValue;

fn main() {
    let mut args = std::env::args();
    args.next();
    let domain = args.next().expect("Please specify a domain name");
    let dns_server = args.next().unwrap_or_else(|| "1.1.1.1".into());

    println!("Resolving {domain} via DNS {dns_server}\n\n");

    let (answers, _) = dns::resolver::resolve_domain(&domain, &dns_server, None, None)
        .expect("Error resolving DNS records");

    for answer in answers {
        let meta = &answer.meta;
        match answer.value {
            AnswerValue::A { ipv4 } => println!("A\t{meta:?} - {ipv4}"),
            AnswerValue::CNAME { cname } => println!("CNAME\t{meta:?} - {cname}"),
            AnswerValue::AAAA { ipv6 } => println!("CNAME\t{meta:?} - {ipv6}"),
            AnswerValue::NS { ns } => println!("CNAME\t{meta:?} - {ns}"),
            AnswerValue::MB { domain_name } => println!("CNAME\t{meta:?} - {domain_name}"),
            AnswerValue::MX {
                preference,
                exchange,
            } => println!("CNAME\t{meta:?} - {exchange} ({preference})"),
            AnswerValue::PTR { domain_name } => println!("CNAME\t{meta:?} - {domain_name}"),
            AnswerValue::SOA {
                mname,
                rname,
                serial: _,
                refresh: _,
                retry: _,
                expire: _,
                minimum: _,
            } => println!("CNAME\t{meta:?} - {mname} - {rname}"),
        }
    }
}
