import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import org.apache.pdfbox.pdfwriter.compress.CompressParameters;
import org.apache.pdfbox.pdmodel.PDDocumentNameDictionary;
import org.apache.pdfbox.pdmodel.common.filespecification.PDComplexFileSpecification;
import org.apache.pdfbox.cos.COSName;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.transform.OutputKeys;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.stream.StreamResult;
import org.apache.pdfbox.Loader;
import org.apache.pdfbox.pdmodel.PDDocument;
import org.apache.pdfbox.pdmodel.common.PDMetadata;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;

/** Removes optional author/software metadata, preserving required PDF/A and invoice XMP. */
public final class BillfluxPdfMetadata {
    private static final String PDF = "http://ns.adobe.com/pdf/1.3/";
    private static final String XMP = "http://ns.adobe.com/xap/1.0/";
    private static final String DC = "http://purl.org/dc/elements/1.1/";

    private static void update(Document document, String namespace, String name, String value) {
        NodeList nodes = document.getElementsByTagNameNS(namespace, name);
        for (int index = nodes.getLength() - 1; index >= 0; index--) {
            Node node = nodes.item(index);
            if (value == null) node.getParentNode().removeChild(node);
            else node.setTextContent(value);
        }
        // XMP also permits simple properties as attributes.
        NodeList elements = document.getElementsByTagName("*");
        for (int index = 0; index < elements.getLength(); index++) {
            Element element = (Element) elements.item(index);
            if (element.hasAttributeNS(namespace, name)) {
                if (value == null) element.removeAttributeNS(namespace, name);
                else element.getAttributeNodeNS(namespace, name).setValue(value);
            }
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("Expected input PDF, output PDF and invoice XML paths");
        try (PDDocument pdf = Loader.loadPDF(new File(args[0]))) {
            PDMetadata metadata = pdf.getDocumentCatalog().getMetadata();
            if (metadata == null) throw new IllegalStateException("PDF/A XMP metadata missing");
            DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
            factory.setNamespaceAware(true);
            factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
            factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "");
            factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");
            Document xmp = factory.newDocumentBuilder().parse(new ByteArrayInputStream(metadata.toByteArray()));
            update(xmp, PDF, "Producer", null);
            update(xmp, XMP, "CreatorTool", null);
            update(xmp, DC, "creator", null);
            pdf.getDocumentInformation().setProducer(null);
            pdf.getDocumentInformation().setCreator(null);
            pdf.getDocumentInformation().setAuthor(null);
            TransformerFactory transformers = TransformerFactory.newInstance();
            transformers.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "");
            transformers.setAttribute(XMLConstants.ACCESS_EXTERNAL_STYLESHEET, "");
            var transformer = transformers.newTransformer();
            transformer.setOutputProperty(OutputKeys.ENCODING, "UTF-8");
            transformer.setOutputProperty(OutputKeys.OMIT_XML_DECLARATION, "yes");
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            transformer.transform(new DOMSource(xmp), new StreamResult(bytes));
            metadata.importXMPMetadata(bytes.toByteArray());
            // Match Mustang's classic PDF object layout for compatibility with readers.
            pdf.save(args[1], CompressParameters.NO_COMPRESSION);
        }
        // Check the saved file, not only the in-memory document.
        try (PDDocument saved = Loader.loadPDF(new File(args[1]))) {
            var tree = new PDDocumentNameDictionary(saved.getDocumentCatalog()).getEmbeddedFiles();
            var names = tree == null ? null : tree.getNames();
            if (names == null || names.size() != 1 || tree.getKids() != null) {
                throw new IllegalStateException("Expected exactly one invoice attachment");
            }
            PDComplexFileSpecification file = names.values().iterator().next();
            var associated = saved.getDocumentCatalog().getCOSObject().getCOSArray(COSName.AF);
            if (associated == null || associated.size() != 1 || associated.getObject(0) != file.getCOSObject()) {
                throw new IllegalStateException("Invoice attachment and associated file disagree");
            }
            byte[] expected = Files.readAllBytes(Path.of(args[2]));
            if (!Arrays.equals(expected, file.getEmbeddedFile().toByteArray()) ||
                !Arrays.equals(expected, file.getEmbeddedFileUnicode().toByteArray())) {
                throw new IllegalStateException("Embedded invoice XML differs from the source");
            }
        }
    }
}
