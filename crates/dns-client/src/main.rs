use dns::protocol::answer::ResourceRecordData;

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
            ResourceRecordData::A { ipv4 } => println!("A\t{meta:?} - {ipv4}"),
            ResourceRecordData::CNAME { cname } => println!("CNAME\t{meta:?} - {cname}"),
            ResourceRecordData::AAAA { ipv6 } => println!("CNAME\t{meta:?} - {ipv6}"),
            ResourceRecordData::NS { ns } => println!("CNAME\t{meta:?} - {ns}"),
            ResourceRecordData::MB { domain_name } => println!("CNAME\t{meta:?} - {domain_name}"),
            ResourceRecordData::MX {
                preference,
                exchange,
            } => println!("CNAME\t{meta:?} - {exchange} ({preference})"),
            ResourceRecordData::PTR { domain_name } => println!("CNAME\t{meta:?} - {domain_name}"),
            ResourceRecordData::SOA {
                mname,
                rname,
                serial: _,
                refresh: _,
                retry: _,
                expire: _,
                minimum: _,
            } => println!("CNAME\t{meta:?} - {mname} - {rname}"),
            ResourceRecordData::Unknown => {
                println!("Unknown record type {:?}", meta.record_type)
            }
        }
    }
}
