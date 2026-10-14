import AppKit
let size = Int(CommandLine.arguments[1])!
let out = CommandLine.arguments[2]
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
let s = CGFloat(size)
NSColor(calibratedRed: 0.122, green: 0.110, blue: 0.086, alpha: 1).setFill()
NSBezierPath(roundedRect: NSRect(x: s*0.08, y: s*0.08, width: s*0.84, height: s*0.84), xRadius: s*0.19, yRadius: s*0.19).fill()
NSColor(calibratedRed: 0.91, green: 0.64, blue: 0.24, alpha: 1).setStroke()
let frame = NSBezierPath(roundedRect: NSRect(x: s*0.23, y: s*0.33, width: s*0.54, height: s*0.37), xRadius: s*0.055, yRadius: s*0.055)
frame.lineWidth = s*0.035
frame.stroke()
let stand = NSBezierPath(); stand.move(to: NSPoint(x: s*0.5,y: s*0.31)); stand.line(to: NSPoint(x: s*0.5,y: s*0.23)); stand.move(to: NSPoint(x: s*0.39,y: s*0.23)); stand.line(to: NSPoint(x: s*0.61,y: s*0.23)); stand.lineWidth = s*0.035; stand.lineCapStyle = .round; stand.stroke()
NSColor(calibratedRed: 0.91, green: 0.64, blue: 0.24, alpha: 1).setFill()
NSBezierPath(ovalIn: NSRect(x:s*0.445,y:s*0.46,width:s*0.11,height:s*0.11)).fill()
NSGraphicsContext.restoreGraphicsState()
try rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: out))
