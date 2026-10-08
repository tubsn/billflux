from pathlib import Path
import sys
import xml.etree.ElementTree as ET
from pypdf import PdfReader

pdf_path, xml_path = map(Path, sys.argv[1:3])
reader = PdfReader(pdf_path)
assert not reader.metadata.producer
assert not reader.metadata.creator
assert not reader.metadata.author
xmp = ET.fromstring(reader.xmp_metadata.stream.get_data())
ns = {'pdf': 'http://ns.adobe.com/pdf/1.3/', 'xmp': 'http://ns.adobe.com/xap/1.0/', 'dc': 'http://purl.org/dc/elements/1.1/', 'pdfaid': 'http://www.aiim.org/pdfa/ns/id/', 'zf': 'urn:zugferd:pdfa:CrossIndustryDocument:invoice:2p0#'}
assert xmp.find('.//pdf:Producer',ns) is None
assert xmp.find('.//xmp:CreatorTool',ns) is None
assert xmp.find('.//dc:creator',ns) is None
assert xmp.find('.//pdfaid:part',ns).text == '3'
assert xmp.find('.//pdfaid:conformance',ns).text == 'U'
assert xmp.find('.//zf:ConformanceLevel',ns).text == 'EN 16931'
name=xmp.find('.//zf:DocumentFileName',ns).text
attachments=list(reader.attachments.items())
assert len(attachments)==1 and attachments[0][0]==name
assert attachments[0][1]==[xml_path.read_bytes()]
assert len(reader.trailer['/Root']['/AF'])==1
assert '${project.version}' not in str(reader.metadata)
assert b'/ObjStm' not in pdf_path.read_bytes()
print('Passed: no software/author metadata, PDF/A and invoice declarations preserved, exactly one byte-identical XML attachment.')
